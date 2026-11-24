//! La machine : son état et la boucle d'exécution (spec B.2).

use pbc::{Fault, Instruction, Program, decode};

use crate::Stack;

/// Ce qui se passe après un pas d'exécution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// L'instruction est exécutée, la machine peut continuer.
    Continue,
    /// `HALT` a été exécuté : fin normale.
    Halted,
}

/// Une machine qui exécute un programme chargé.
#[derive(Debug, Clone)]
pub struct Machine<'a> {
    program: &'a Program,
    pc: usize,
    gas: u16,
    stack: Stack,
}

impl<'a> Machine<'a> {
    /// Prépare une machine au début du programme, avec le budget de gaz donné.
    pub fn new(program: &'a Program, gas_budget: u16) -> Self {
        Self {
            program,
            pc: 0,
            gas: gas_budget,
            stack: Stack::new(),
        }
    }

    /// Exécute une seule instruction : décoder, payer, exécuter, avancer.
    pub fn step(&mut self) -> Result<Step, Fault> {
        // 1. Décoder l'instruction au pc (PcOutOfRange, BadOpcode, Truncated).
        let instruction = decode(self.program.code(), self.pc)?;

        // 2. Payer le gaz. S'il n'y en a pas assez : OutOfGas, sans rien exécuter.
        self.gas = self
            .gas
            .checked_sub(gas_cost(instruction))
            .ok_or(Fault::OutOfGas)?;

        // 3. Exécuter.
        match instruction {
            Instruction::Halt => return Ok(Step::Halted),
            Instruction::PushI16(n) => self.stack.push(i64::from(n))?,
            Instruction::Add => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(a.wrapping_add(b))?;
            }
            // Temporaire : les autres instructions arrivent aux prochaines étapes.
            _ => return Err(Fault::BadOpcode),
        }

        // 4. Avancer à l'instruction suivante.
        self.pc += instruction.size();
        Ok(Step::Continue)
    }

    /// Exécute jusqu'à `HALT` (Ok) ou jusqu'à une faute (Err).
    pub fn run(&mut self) -> Result<(), Fault> {
        loop {
            if self.step()? == Step::Halted {
                return Ok(());
            }
        }
    }

    /// La pile, en lecture seule.
    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    /// Le gaz qui reste.
    pub fn gas(&self) -> u16 {
        self.gas
    }
}

/// Prix en gaz de chaque instruction (table de l'annexe B).
fn gas_cost(instruction: Instruction) -> u16 {
    match instruction {
        Instruction::Halt => 0,
        Instruction::PushI16(_) => 1,
        Instruction::Add => 2,
        // Temporaire : complété avec les autres instructions.
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fabrique un programme .pbc sans constantes autour du code donné.
    fn programme(code: &[u8]) -> Program {
        let mut bytes = vec![0x50, 0x42, 0x43, 0x31, 0x01, 0x00, 0x00, 0x00];
        bytes.extend_from_slice(&(code.len() as u32).to_be_bytes());
        bytes.extend_from_slice(code);
        Program::load(&bytes).unwrap()
    }

    #[test]
    fn trois_plus_quatre_donne_sept() {
        // PUSHI16 3, PUSHI16 4, ADD, HALT
        let p = programme(&[0x02, 0x00, 0x03, 0x02, 0x00, 0x04, 0x10, 0x00]);
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Ok(()));
        assert_eq!(m.stack().len(), 1);
        assert_eq!(m.stack().clone().pop(), Ok(7));
        assert_eq!(m.gas(), 1000 - 1 - 1 - 2);
    }

    #[test]
    fn plus_assez_de_gaz_donne_out_of_gas_sans_payer() {
        // PUSHI16 3, PUSHI16 4, ADD : il faut 4 de gaz, on en donne 3.
        let p = programme(&[0x02, 0x00, 0x03, 0x02, 0x00, 0x04, 0x10, 0x00]);
        let mut m = Machine::new(&p, 3);
        assert_eq!(m.run(), Err(Fault::OutOfGas));
        assert_eq!(m.gas(), 1);
    }

    #[test]
    fn add_sur_une_seule_valeur_donne_stack_underflow() {
        let p = programme(&[0x02, 0x00, 0x03, 0x10, 0x00]);
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Err(Fault::StackUnderflow));
    }

    #[test]
    fn add_enveloppe_au_lieu_de_deborder() {
        let p = programme(&[0x02, 0x00, 0x01, 0x10, 0x00]);
        let mut m = Machine::new(&p, 1000);
        m.stack.push(i64::MAX).unwrap();
        assert_eq!(m.run(), Ok(()));
        assert_eq!(m.stack().clone().pop(), Ok(i64::MIN));
    }
}
