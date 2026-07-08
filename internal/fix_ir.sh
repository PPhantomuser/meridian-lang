sed -i '' -e 's/next_reg: None,/next_reg: 0,/g' ir/src/lib.rs
sed -i '' -e 's/self.next_reg = None;/self.next_reg = 0;/g' ir/src/lib.rs
sed -i '' -e 's/Opcode::JumpIfFalse(cond_reg, None)/Opcode::JumpIfFalse(cond_reg, 0)/g' ir/src/lib.rs
sed -i '' -e 's/1.None/1.0/g' ir/src/lib.rs
sed -i '' -e 's/Opcode::Jump(None)/Opcode::Jump(0)/g' ir/src/lib.rs
sed -i '' -e 's/if i > None/if i > 0/g' ir/src/lib.rs
sed -i '' -e 's/(compiled_fn, true, None)/(compiled_fn, true, 0)/g' ir/src/lib.rs
sed -i '' -e 's/Opcode::AsyncCall(dest, synthetic_name, None, None)/Opcode::AsyncCall(dest, synthetic_name, 0, 0)/g' ir/src/lib.rs
