//! This module is responsible for generating the vm instruction flowgraph.

use crate::vm::{CodeBlock, Opcode};
use boa_interner::{Interner, Sym};
use std::mem::size_of;

mod color;
mod edge;
mod graph;
mod node;

pub use color::*;
pub use edge::*;
pub use graph::*;
pub use node::*;

impl CodeBlock {
    /// Output the [`CodeBlock`] VM instructions into a [`Graph`].
    #[inline]
    pub fn to_graph(&self, interner: &Interner, graph: &mut SubGraph) {
        // Have to remove any invalid graph chars like `<` or `>`.
        let name = if self.name == Sym::MAIN {
            "__main__".to_string()
        } else {
            interner.resolve_expect(self.name).to_string()
        };

        graph.set_label(name);

        let mut environments = Vec::new();
        let mut try_entries = Vec::new();
        let mut returns = Vec::new();

        let mut pc = 0;
        while pc < self.code.len() {
            let opcode: Opcode = self.code[pc].try_into().expect("invalid opcode");
            let opcode_str = opcode.as_str();
            let previous_pc = pc;

            let mut tmp = pc;
            let label = format!(
                "{opcode_str} {}",
                self.instruction_operands(&mut tmp, interner)
            );

            pc += size_of::<Opcode>();
            match opcode {
                Opcode::SetFunctionName => {
                    let operand = self.read::<u8>(pc);
                    pc += size_of::<u8>();
                    let label = format!(
                        "{opcode_str} {}",
                        match operand {
                            0 => "prefix: none",
                            1 => "prefix: get",
                            2 => "prefix: set",
                            _ => unreachable!(),
                        }
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::RotateLeft | Opcode::RotateRight => {
                    pc += size_of::<u8>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushInt8 => {
                    pc += size_of::<i8>();

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushInt16 => {
                    pc += size_of::<i16>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushInt32 => {
                    pc += size_of::<i32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushRational => {
                    pc += size_of::<f64>();

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushLiteral => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let operand_str = self.literals[operand as usize].display().to_string();
                    let operand_str = operand_str.escape_debug();
                    let label = format!("{opcode_str} {}", operand_str);

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::Jump => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        operand as usize,
                        None,
                        Color::None,
                        EdgeStyle::Line,
                    );
                }
                Opcode::JumpIfFalse
                | Opcode::JumpIfNotUndefined
                | Opcode::JumpIfNullOrUndefined => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        operand as usize,
                        Some("YES".into()),
                        Color::Green,
                        EdgeStyle::Line,
                    );
                    graph.add_edge(
                        previous_pc,
                        pc,
                        Some("NO".into()),
                        Color::Red,
                        EdgeStyle::Line,
                    );
                }
                Opcode::LogicalAnd | Opcode::LogicalOr | Opcode::Coalesce => {
                    let exit = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        exit as usize,
                        Some("SHORT CIRCUIT".into()),
                        Color::Red,
                        EdgeStyle::Line,
                    );
                }
                Opcode::Case => {
                    let address = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        pc,
                        Some("NO".into()),
                        Color::Red,
                        EdgeStyle::Line,
                    );
                    graph.add_edge(
                        previous_pc,
                        address,
                        Some("YES".into()),
                        Color::Green,
                        EdgeStyle::Line,
                    );
                }
                Opcode::Default => {
                    let address = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, address, None, Color::None, EdgeStyle::Line);
                }
                Opcode::ForInLoopInitIterator => {
                    let address = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        address,
                        Some("NULL OR UNDEFINED".into()),
                        Color::None,
                        EdgeStyle::Line,
                    );
                }
                Opcode::ForInLoopNext
                | Opcode::ForAwaitOfLoopNext
                | Opcode::GeneratorNextDelegate => {
                    let address = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        address,
                        Some("DONE".into()),
                        Color::None,
                        EdgeStyle::Line,
                    );
                }
                Opcode::CatchStart
                | Opcode::FinallySetJump
                | Opcode::CallEval
                | Opcode::Call
                | Opcode::New
                | Opcode::SuperCall
                | Opcode::ConcatToString => {
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::TryStart => {
                    let next_address = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let finally_address = self.read::<u32>(pc);
                    pc += size_of::<u32>();

                    try_entries.push((
                        previous_pc,
                        next_address,
                        if finally_address == 0 {
                            None
                        } else {
                            Some(finally_address)
                        },
                    ));

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        next_address as usize,
                        Some("NEXT".into()),
                        Color::None,
                        EdgeStyle::Line,
                    );
                    if finally_address != 0 {
                        graph.add_edge(
                            previous_pc,
                            finally_address as usize,
                            Some("FINALLY".into()),
                            Color::None,
                            EdgeStyle::Line,
                        );
                    }
                }
                Opcode::CopyDataProperties => {
                    let operand1 = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let operand2 = self.read::<u32>(pc);
                    pc += size_of::<u32>();

                    let label = format!("{opcode_str} {operand1}, {operand2}");
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushDeclarativeEnvironment | Opcode::PushFunctionEnvironment => {
                    let random = rand::random();
                    environments.push((previous_pc, random));

                    pc += size_of::<u32>();
                    pc += size_of::<u32>();

                    graph.add_node(
                        previous_pc,
                        NodeShape::None,
                        label.into(),
                        Color::from_random_number(random),
                    );
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PopEnvironment => {
                    let (environment_push, random) = environments
                        .pop()
                        .expect("There should be a push evironment before");

                    let color = Color::from_random_number(random);
                    graph.add_node(previous_pc, NodeShape::None, label.into(), color);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph
                        .add_edge(
                            previous_pc,
                            environment_push,
                            None,
                            color,
                            EdgeStyle::Dotted,
                        )
                        .set_type(EdgeType::None);
                }
                Opcode::GetArrowFunction
                | Opcode::GetAsyncArrowFunction
                | Opcode::GetFunction
                | Opcode::GetFunctionAsync
                | Opcode::GetGenerator
                | Opcode::GetGeneratorAsync => {
                    let operand = self.read::<u32>(pc);
                    let fn_name = interner
                        .resolve_expect(self.functions[operand as usize].name)
                        .to_string();
                    pc += size_of::<u32>();
                    let label = format!(
                        "{opcode_str} '{fn_name}' (length: {})",
                        self.functions[operand as usize].length
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::DefInitArg
                | Opcode::DefVar
                | Opcode::DefInitVar
                | Opcode::DefLet
                | Opcode::DefInitLet
                | Opcode::DefInitConst
                | Opcode::GetName
                | Opcode::GetNameOrUndefined
                | Opcode::SetName
                | Opcode::DeleteName => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let label = format!(
                        "{opcode_str} '{}'",
                        interner.resolve_expect(self.bindings[operand as usize].name().sym()),
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::GetPropertyByName
                | Opcode::SetPropertyByName
                | Opcode::DefineOwnPropertyByName
                | Opcode::DefineClassMethodByName
                | Opcode::SetPropertyGetterByName
                | Opcode::DefineClassGetterByName
                | Opcode::SetPropertySetterByName
                | Opcode::DefineClassSetterByName
                | Opcode::AssignPrivateField
                | Opcode::SetPrivateField
                | Opcode::SetPrivateMethod
                | Opcode::SetPrivateSetter
                | Opcode::SetPrivateGetter
                | Opcode::GetPrivateField
                | Opcode::DeletePropertyByName
                | Opcode::PushClassFieldPrivate
                | Opcode::PushClassPrivateGetter
                | Opcode::PushClassPrivateSetter
                | Opcode::PushClassPrivateMethod => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let label = format!(
                        "{opcode_str} '{}'",
                        interner.resolve_expect(self.names[operand as usize].sym()),
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::Throw => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    if let Some((_try_pc, next, _finally)) = try_entries.last() {
                        graph.add_edge(
                            previous_pc,
                            *next as usize,
                            Some("CAUGHT".into()),
                            Color::None,
                            EdgeStyle::Line,
                        );
                    }
                }
                Opcode::Pop
                | Opcode::PopIfThrown
                | Opcode::Dup
                | Opcode::Swap
                | Opcode::PushZero
                | Opcode::PushOne
                | Opcode::PushNaN
                | Opcode::PushPositiveInfinity
                | Opcode::PushNegativeInfinity
                | Opcode::PushNull
                | Opcode::PushTrue
                | Opcode::PushFalse
                | Opcode::PushUndefined
                | Opcode::PushEmptyObject
                | Opcode::PushClassPrototype
                | Opcode::SetClassPrototype
                | Opcode::SetHomeObject
                | Opcode::Add
                | Opcode::Sub
                | Opcode::Div
                | Opcode::Mul
                | Opcode::Mod
                | Opcode::Pow
                | Opcode::ShiftRight
                | Opcode::ShiftLeft
                | Opcode::UnsignedShiftRight
                | Opcode::BitOr
                | Opcode::BitAnd
                | Opcode::BitXor
                | Opcode::BitNot
                | Opcode::In
                | Opcode::Eq
                | Opcode::StrictEq
                | Opcode::NotEq
                | Opcode::StrictNotEq
                | Opcode::GreaterThan
                | Opcode::GreaterThanOrEq
                | Opcode::LessThan
                | Opcode::LessThanOrEq
                | Opcode::InstanceOf
                | Opcode::TypeOf
                | Opcode::Void
                | Opcode::LogicalNot
                | Opcode::Pos
                | Opcode::Neg
                | Opcode::Inc
                | Opcode::IncPost
                | Opcode::Dec
                | Opcode::DecPost
                | Opcode::GetPropertyByValue
                | Opcode::GetPropertyByValuePush
                | Opcode::SetPropertyByValue
                | Opcode::DefineOwnPropertyByValue
                | Opcode::DefineClassMethodByValue
                | Opcode::SetPropertyGetterByValue
                | Opcode::DefineClassGetterByValue
                | Opcode::SetPropertySetterByValue
                | Opcode::DefineClassSetterByValue
                | Opcode::DeletePropertyByValue
                | Opcode::DeleteSuperThrow
                | Opcode::ToPropertyKey
                | Opcode::ToBoolean
                | Opcode::CatchEnd
                | Opcode::CatchEnd2
                | Opcode::FinallyStart
                | Opcode::FinallyEnd
                | Opcode::This
                | Opcode::Super
                | Opcode::LoopStart
                | Opcode::LoopContinue
                | Opcode::LoopEnd
                | Opcode::InitIterator
                | Opcode::InitIteratorAsync
                | Opcode::IteratorNext
                | Opcode::IteratorClose
                | Opcode::IteratorToArray
                | Opcode::RequireObjectCoercible
                | Opcode::ValueNotNullOrUndefined
                | Opcode::RestParameterInit
                | Opcode::RestParameterPop
                | Opcode::PushValueToArray
                | Opcode::PushElisionToArray
                | Opcode::PushIteratorToArray
                | Opcode::PushNewArray
                | Opcode::PopOnReturnAdd
                | Opcode::PopOnReturnSub
                | Opcode::Yield
                | Opcode::GeneratorNext
                | Opcode::AsyncGeneratorNext
                | Opcode::PushClassField
                | Opcode::SuperCallDerived
                | Opcode::Await
                | Opcode::PushNewTarget
                | Opcode::CallEvalSpread
                | Opcode::CallSpread
                | Opcode::NewSpread
                | Opcode::SuperCallSpread
                | Opcode::ForAwaitOfLoopIterate
                | Opcode::SetPrototype
                | Opcode::Nop => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::TryEnd => {
                    try_entries
                        .pop()
                        .expect("there should already be try block");

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::Return => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    if let Some((_try_pc, _next, Some(finally))) = try_entries.last() {
                        graph.add_edge(
                            previous_pc,
                            *finally as usize,
                            None,
                            Color::None,
                            EdgeStyle::Line,
                        );
                    } else {
                        returns.push(previous_pc);
                    }
                }
            }
        }

        for ret in returns {
            graph.add_edge(ret, pc, None, Color::None, EdgeStyle::Line);
        }

        graph.add_node(pc, NodeShape::Diamond, "End".into(), Color::Red);

        for function in &self.functions {
            let subgraph = graph.subgraph(String::new());
            function.to_graph(interner, subgraph);
        }
    }
}
