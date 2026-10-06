use crate::ast::Block;
use crate::frontend::cfg::function::{
    Const, Function, Instruction, Label, Pointer, Terminator, Var,
};
use crate::frontend::group::Group;
use crate::frontend::namespace::Namespace;
use crate::frontend::stage::II;
use crate::optimizer::bytecode::{Bytecode, Id, Index, Reg};
use crate::optimizer::codegen::copies::Copy;
use crate::optimizer::stage::{Callable, III};
use crate::philia093::Interner;
use std::collections::HashMap;

struct Context {
    data: Data,
    groups: Worker<Group>,
    functions: Worker<(Vec<usize>, Block)>,
}

impl Context {
    fn new(groups: HashMap<usize, Group>, functions: HashMap<usize, (Vec<usize>, Block)>) -> Self {
        Self {
            data: Data {
                pool: vec![],
                fast: Default::default(),
            },
            groups: Worker {
                indices: Default::default(),
                source: groups,
                worklist: vec![],
            },
            functions: Worker {
                indices: Default::default(),
                source: functions,
                worklist: vec![],
            },
        }
    }

    fn done(&self) -> bool {
        self.groups.worklist.is_empty() && self.functions.worklist.is_empty()
    }
}

struct Data {
    pool: Vec<Const>,
    fast: HashMap<Const, usize>,
}

impl Data {
    fn index(&mut self, key: Const) -> usize {
        if let Some(&id) = self.fast.get(&key) {
            return id;
        }
        let id = self.pool.len();
        self.fast.insert(key.clone(), id);
        self.pool.push(key);
        id
    }
}

struct Worker<T> {
    indices: HashMap<usize, Index>,
    source: HashMap<usize, T>,
    worklist: Vec<(Index, T)>,
}

impl<T> Worker<T> {
    fn get(&mut self, id: usize) -> Index {
        if let Some(index) = self.indices.get(&id) {
            return *index;
        }
        let index = Index::try_from(self.indices.len()).unwrap();
        self.indices.insert(id, index);
        let todo = self.source.remove(&id).unwrap();
        self.worklist.push((index, todo));
        index
    }

    fn pop(&mut self) -> Option<(Index, T)> {
        self.worklist.pop()
    }
}

impl II {
    pub fn codegen(self, limit: usize) -> Result<III, String> {
        let mut context = Context::new(self.groups, self.functions);
        let mut groups = HashMap::new();
        let mut callables = HashMap::new();

        let main = compile(
            vec![self.main.0],
            self.main.1,
            limit,
            &self.interner,
            &self.namespace,
            &mut context,
        )?;

        while !context.done() {
            while let Some((index, mut group)) = context.groups.pop() {
                for id in group.methods.values_mut() {
                    *id = context.functions.get(*id as usize)
                }
                groups.insert(index, group);
            }

            while let Some((index, (args, block))) = context.functions.pop() {
                let callable = compile(
                    args,
                    block,
                    limit,
                    &self.interner,
                    &self.namespace,
                    &mut context,
                )?;
                callables.insert(index, callable);
            }
        }

        Ok(III {
            main,
            text: linearize(callables),
            data: context.data.pool,
            groups: linearize(groups),
        })
    }
}

fn compile(
    args: Vec<usize>,
    block: Block,
    limit: usize,
    interner: &Interner,
    namespace: &Namespace,
    ctx: &mut Context,
) -> Result<Callable, String> {
    let length = Reg::try_from(args.len()).unwrap();
    let map = block
        .semantic(args.iter(), namespace)
        .map_err(|e| e.recover(interner))?;
    let mut function = block
        .function(&map, interner, args)
        .map_err(|e| e.recover(interner))?;
    function.optimize(limit)?;
    let copies = function.copies();
    let rpo = function.rpo();
    let (allocation, used) = function.allocate(&rpo, &copies);
    Ok(Callable {
        args: length,
        registers: used,
        bytecodes: function.codegen(&rpo, &allocation, ctx, copies),
    })
}

fn linearize<T>(mut map: HashMap<Index, T>) -> Vec<T> {
    let mut i = 0;
    let mut all = Vec::new();
    while let Some(value) = map.remove(&i) {
        all.push(value);
        i += 1;
    }
    all
}

impl Function {
    fn codegen(
        &mut self,
        rpo: &[Label],
        alloc: &HashMap<Var, Reg>,
        ctx: &mut Context,
        mut copies: HashMap<Label, Vec<Copy>>,
    ) -> Vec<Bytecode> {
        let mut index = 0;
        let mut map = HashMap::new();
        for label in rpo {
            map.insert(*label, index);
            let fragment = self.get(*label).unwrap();
            let body = fragment.instructions.len();
            let copy = copies.get(label).map(|x| x.len()).unwrap_or(0);
            let term = fragment.terminator.is_some() as usize;
            index += copy + body + term
        }

        let mut bytecodes = Vec::with_capacity(index);
        for label in rpo {
            let fragment = self.get(*label).unwrap();

            let body = fragment.instructions.iter().map(|x| x.codegen(alloc, ctx));
            bytecodes.extend(body);

            let copy = copies
                .remove(label)
                .unwrap_or_default()
                .into_iter()
                .map(|copy| copy.codegen(alloc));
            bytecodes.extend(copy);

            if let Some(term) = fragment.terminator.as_ref().map(|x| x.codegen(alloc, &map)) {
                bytecodes.push(term);
            }
        }
        bytecodes
    }
}

impl Instruction {
    fn codegen(&self, alloc: &HashMap<Var, Reg>, ctx: &mut Context) -> Bytecode {
        match self {
            Instruction::Arg(dst, idx) => Bytecode::Arg(alloc[dst], Index::try_from(*idx).unwrap()),
            Instruction::Field(dst, src, id) => {
                Bytecode::Field(alloc[dst], alloc[src], Id::try_from(*id).unwrap())
            }
            Instruction::Unpack(dst, src, idx) => {
                Bytecode::Unpack(alloc[dst], alloc[src], Index::try_from(*idx).unwrap())
            }
            Instruction::Pointer(dst, pt, ptr) => match pt {
                Pointer::Function => Bytecode::Pointer(
                    alloc[dst],
                    *pt,
                    Index::try_from(ctx.functions.get(*ptr)).unwrap(),
                ),
                Pointer::Group => Bytecode::Pointer(
                    alloc[dst],
                    *pt,
                    Index::try_from(ctx.groups.get(*ptr)).unwrap(),
                ),
                Pointer::Rust => Bytecode::Pointer(alloc[dst], *pt, Index::try_from(*ptr).unwrap()),
            },
            Instruction::Load(dst, id) => Bytecode::Load(
                alloc[dst],
                Index::try_from(ctx.data.index(id.clone())).unwrap(),
            ),
            Instruction::Binary(dst, lhs, op, rhs) => Bytecode::Binary(
                alloc[dst],
                *alloc.get(lhs).unwrap_or(&0),
                *op,
                *alloc.get(rhs).unwrap_or(&0),
            ),
            Instruction::Unary(dst, op, src) => Bytecode::Unary(alloc[dst], *op, alloc[src]),
            Instruction::Call(dst, src, args) => Bytecode::Call(
                alloc[dst],
                alloc[src],
                args.iter().map(|x| alloc[x]).collect(),
            ),
            Instruction::List(dst, args) => {
                Bytecode::List(alloc[dst], args.iter().map(|x| alloc[x]).collect())
            }
            Instruction::Tuple(dst, args) => {
                Bytecode::Tuple(alloc[dst], args.iter().map(|x| alloc[x]).collect())
            }
            Instruction::Index(dst, src, index) => {
                Bytecode::Index(alloc[dst], alloc[src], alloc[index])
            }
            Instruction::Method(dst, src, id, args) => Bytecode::Method(
                alloc[dst],
                alloc[src],
                Id::try_from(*id).unwrap(),
                args.iter().map(|x| alloc[x]).collect(),
            ),
        }
    }
}

impl Terminator {
    fn codegen(&self, alloc: &HashMap<Var, Reg>, map: &HashMap<Label, usize>) -> Bytecode {
        match self {
            Terminator::Branch(cond, yes, no) => Bytecode::Branch(
                alloc[cond],
                Index::try_from(map[yes]).unwrap(),
                Index::try_from(map[no]).unwrap(),
            ),
            Terminator::Jump(to) => Bytecode::Jump(Index::try_from(map[to]).unwrap()),
            Terminator::Return(var) => Bytecode::Return(alloc[var]),
        }
    }
}

impl Copy {
    fn codegen(&self, alloc: &HashMap<Var, Reg>) -> Bytecode {
        Bytecode::Copy(alloc[&self.0], alloc[&self.1])
    }
}
