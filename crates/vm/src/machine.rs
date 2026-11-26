//! La machine : son état et la boucle d'exécution (spec B.2).

use pbc::{Fault, Instruction, Program, decode};

use crate::Stack;

/// Nombre de cases de la mémoire (spec : 256).
pub const MEMORY_SIZE: usize = 256;

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
    memory: [i64; MEMORY_SIZE],
}

impl<'a> Machine<'a> {
    /// Prépare une machine au début du programme, avec le budget de gaz donné.
    pub fn new(program: &'a Program, gas_budget: u16) -> Self {
        Self {
            program,
            pc: 0,
            gas: gas_budget,
            stack: Stack::new(),
            memory: [0; MEMORY_SIZE],
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
            // a b -- a-b
            Instruction::Sub => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(a.wrapping_sub(b))?;
            }
            // a b -- a*b
            Instruction::Mul => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(a.wrapping_mul(b))?;
            }
            // a b -- a/b
            Instruction::Div => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                if b == 0 {
                    return Err(Fault::DivByZero);
                }
                self.stack.push(a.wrapping_div(b))?;
            }
            // a b -- a%b
            Instruction::Mod => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                if b == 0 {
                    return Err(Fault::DivByZero);
                }
                self.stack.push(a.wrapping_rem(b))?;
            }
            // a -- -a
            Instruction::Neg => {
                let a = self.stack.pop()?;
                self.stack.push(a.wrapping_neg())?;
            }
            // a b -- 0|1
            Instruction::Eq => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(i64::from(a == b))?;
            }
            // a b -- 0|1
            Instruction::Lt => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(i64::from(a < b))?;
            }
            // a b -- 0|1
            Instruction::Gt => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(i64::from(a > b))?;
            }
            // a -- 0|1 (logique : 1 si a vaut 0)
            Instruction::Not => {
                let a = self.stack.pop()?;
                self.stack.push(i64::from(a == 0))?;
            }
            // a b -- a&b
            Instruction::And => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(a & b)?;
            }
            // a b -- a|b
            Instruction::Or => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(a | b)?;
            }
            // a b -- a^b
            Instruction::Xor => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(a ^ b)?;
            }
            // a -- !a (bit à bit : chaque bit est inversé)
            Instruction::BNot => {
                let a = self.stack.pop()?;
                self.stack.push(!a)?;
            }
            // a n -- v (décalage logique à gauche, B.6)
            Instruction::Shl => {
                let n = self.stack.pop()?;
                let a = self.stack.pop()?;
                let v = match shift_count(n) {
                    Some(n) => ((a as u64) << n) as i64,
                    None => 0,
                };
                self.stack.push(v)?;
            }
            // a n -- v (décalage logique à droite, B.6)
            Instruction::Shr => {
                let n = self.stack.pop()?;
                let a = self.stack.pop()?;
                let v = match shift_count(n) {
                    Some(n) => ((a as u64) >> n) as i64,
                    None => 0,
                };
                self.stack.push(v)?;
            }
            // a n -- v (décalage arithmétique à droite, B.6)
            Instruction::Sar => {
                let n = self.stack.pop()?;
                let a = self.stack.pop()?;
                let v = match shift_count(n) {
                    Some(n) => a >> n,
                    None if a < 0 => -1,
                    None => 0,
                };
                self.stack.push(v)?;
            }
            // -- v (adresse fixe, écrite dans l'instruction)
            Instruction::Load(k) => {
                let v = *self.cell(i64::from(k))?;
                self.stack.push(v)?;
            }
            // v -- (adresse fixe, écrite dans l'instruction)
            Instruction::Store(k) => {
                let v = self.stack.pop()?;
                *self.cell(i64::from(k))? = v;
            }
            // addr -- v (adresse prise sur la pile)
            Instruction::LoadX => {
                let addr = self.stack.pop()?;
                let v = *self.cell(addr)?;
                self.stack.push(v)?;
            }
            // v addr -- (adresse prise sur la pile)
            Instruction::StoreX => {
                let addr = self.stack.pop()?;
                let v = self.stack.pop()?;
                *self.cell(addr)? = v;
            }
            // -- (saut relatif, toujours)
            Instruction::Jmp(d) => {
                self.pc = self.jump_target(instruction.size(), d)?;
                return Ok(Step::Continue);
            }
            // c -- (saute seulement si c vaut 0)
            Instruction::Jz(d) => {
                let c = self.stack.pop()?;
                if c == 0 {
                    self.pc = self.jump_target(instruction.size(), d)?;
                    return Ok(Step::Continue);
                }
            }
            // c -- (saute seulement si c ne vaut pas 0)
            Instruction::Jnz(d) => {
                let c = self.stack.pop()?;
                if c != 0 {
                    self.pc = self.jump_target(instruction.size(), d)?;
                    return Ok(Step::Continue);
                }
            }
            // -- (le panneau « arrivée » : ne fait rien)
            Instruction::JumpDest => {}
            // a --
            Instruction::Drop => {
                self.stack.pop()?;
            }
            // a -- a a
            Instruction::Dup => {
                let a = self.stack.pop()?;
                self.stack.push(a)?;
                self.stack.push(a)?;
            }
            // a b -- b a
            Instruction::Swap => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(b)?;
                self.stack.push(a)?;
            }
            // a b -- a b a
            Instruction::Over => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(a)?;
                self.stack.push(b)?;
                self.stack.push(a)?;
            }
            // a b c -- b c a
            Instruction::Rot => {
                let c = self.stack.pop()?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(b)?;
                self.stack.push(c)?;
                self.stack.push(a)?;
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

    /// Cible d'un saut relatif : `pc + taille + d`, comptée depuis l'instruction suivante (B.4).
    fn jump_target(&self, size: usize, d: i16) -> Result<usize, Fault> {
        let target = self.pc as i64 + size as i64 + i64::from(d);
        self.check_target(target)
    }

    /// Vérifie une cible de saut, dans l'ordre de B.4 : dans le code, puis sur un `JUMPDEST`.
    fn check_target(&self, target: i64) -> Result<usize, Fault> {
        let index = usize::try_from(target).map_err(|_| Fault::BadJump)?;
        match self.program.code().get(index) {
            Some(0x43) => Ok(index),
            _ => Err(Fault::BadJump),
        }
    }

    /// La case mémoire à l'adresse donnée, ou `OutOfBounds` si elle n'existe pas.
    fn cell(&mut self, addr: i64) -> Result<&mut i64, Fault> {
        let index = usize::try_from(addr).map_err(|_| Fault::OutOfBounds)?;
        self.memory.get_mut(index).ok_or(Fault::OutOfBounds)
    }

    /// La mémoire, en lecture seule.
    pub fn memory(&self) -> &[i64; MEMORY_SIZE] {
        &self.memory
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

/// Nombre de bits d'un décalage, s'il est dans `0..=63`. Sinon `None` (B.6).
fn shift_count(n: i64) -> Option<u32> {
    if (0..=63).contains(&n) {
        Some(n as u32)
    } else {
        None
    }
}

/// Prix en gaz de chaque instruction (table de l'annexe B).
fn gas_cost(instruction: Instruction) -> u16 {
    match instruction {
        Instruction::Halt => 0,
        Instruction::PushI16(_) => 1,
        Instruction::Drop => 1,
        Instruction::Dup => 1,
        Instruction::Swap => 1,
        Instruction::Over => 1,
        Instruction::Rot => 1,
        Instruction::Add => 2,
        Instruction::Sub => 2,
        Instruction::Mul => 2,
        Instruction::Div => 7,
        Instruction::Mod => 7,
        Instruction::Neg => 2,
        Instruction::Eq => 2,
        Instruction::Lt => 2,
        Instruction::Gt => 2,
        Instruction::Not => 2,
        Instruction::And => 2,
        Instruction::Or => 2,
        Instruction::Xor => 4,
        Instruction::BNot => 2,
        Instruction::Shl => 3,
        Instruction::Shr => 6,
        Instruction::Sar => 6,
        Instruction::LoadX => 4,
        Instruction::StoreX => 5,
        Instruction::Load(_) => 3,
        Instruction::Store(_) => 4,
        Instruction::Jmp(_) => 2,
        Instruction::Jz(_) => 2,
        Instruction::Jnz(_) => 2,
        Instruction::JumpDest => 1,
        // Temporaire : complété avec les autres instructions.
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// j'ai codé ca à la maain eheh : )
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

    /// Lit toute la pile, du bas vers le sommet (pour les tests).
    fn contenu(m: &Machine) -> Vec<i64> {
        let mut pile = m.stack().clone();
        let mut valeurs = Vec::new();
        while let Ok(v) = pile.pop() {
            valeurs.push(v);
        }
        valeurs.reverse();
        valeurs
    }

    #[test]
    fn manipulations_de_pile() {
        // PUSHI16 1, PUSHI16 2, PUSHI16 3, puis l'instruction testée, puis HALT.
        let cas = [
            (0x03, vec![1, 2]),       // DROP : a --
            (0x04, vec![1, 2, 3, 3]), // DUP  : a -- a a
            (0x05, vec![1, 3, 2]),    // SWAP : a b -- b a
            (0x06, vec![1, 2, 3, 2]), // OVER : a b -- a b a
            (0x07, vec![2, 3, 1]),    // ROT  : a b c -- b c a
        ];
        for (opcode, attendu) in cas {
            let p = programme(&[
                0x02, 0x00, 0x01, 0x02, 0x00, 0x02, 0x02, 0x00, 0x03, opcode, 0x00,
            ]);
            let mut m = Machine::new(&p, 1000);
            assert_eq!(m.run(), Ok(()), "opcode {opcode:#04x}");
            assert_eq!(contenu(&m), attendu, "opcode {opcode:#04x}");
            assert_eq!(m.gas(), 1000 - 4, "opcode {opcode:#04x}");
        }
    }

    #[test]
    fn over_sur_une_seule_valeur_donne_stack_underflow() {
        // Cas 010 du corpus : PUSHI16 1, OVER.
        let p = programme(&[0x02, 0x00, 0x01, 0x06, 0x00]);
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Err(Fault::StackUnderflow));
        assert_eq!(m.gas(), 1000 - 2);
    }

    /// Pose `a` puis `b` sur la pile, exécute l'opcode donné puis `HALT`, et renvoie le sommet.
    fn calcule(a: i64, b: i64, opcode: u8) -> Result<i64, Fault> {
        let p = programme(&[opcode, 0x00]);
        let mut m = Machine::new(&p, 1000);
        m.stack.push(a).unwrap();
        m.stack.push(b).unwrap();
        m.run()?;
        m.stack.pop()
    }

    #[test]
    fn calculs() {
        assert_eq!(calcule(7, 3, 0x11), Ok(4)); // SUB : 7 - 3
        assert_eq!(calcule(7, 3, 0x12), Ok(21)); // MUL
        assert_eq!(calcule(7, 2, 0x13), Ok(3)); // DIV : on garde la partie entière
        assert_eq!(calcule(-7, 2, 0x13), Ok(-3)); // DIV : arrondi vers zéro, comme en Rust
        assert_eq!(calcule(7, 3, 0x14), Ok(1)); // MOD
        assert_eq!(calcule(-7, 3, 0x14), Ok(-1)); // MOD : garde le signe de a
    }

    #[test]
    fn division_par_zero() {
        assert_eq!(calcule(7, 0, 0x13), Err(Fault::DivByZero));
        assert_eq!(calcule(7, 0, 0x14), Err(Fault::DivByZero));
    }

    #[test]
    fn les_cas_extremes_enveloppent() {
        assert_eq!(calcule(i64::MIN, 1, 0x11), Ok(i64::MAX)); // SUB
        assert_eq!(calcule(i64::MIN, -1, 0x13), Ok(i64::MIN)); // DIV (spec, préambule)
        assert_eq!(calcule(i64::MIN, -1, 0x14), Ok(0)); // MOD (spec, préambule)
    }

    #[test]
    fn neg() {
        let p = programme(&[0x02, 0x00, 0x05, 0x15, 0x00]); // PUSHI16 5, NEG, HALT
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Ok(()));
        assert_eq!(m.stack().clone().pop(), Ok(-5));
        assert_eq!(m.gas(), 1000 - 1 - 2);
    }

    #[test]
    fn comparaisons() {
        assert_eq!(calcule(3, 3, 0x20), Ok(1)); // EQ : 3 == 3
        assert_eq!(calcule(3, 4, 0x20), Ok(0)); // EQ : 3 == 4
        assert_eq!(calcule(3, 4, 0x21), Ok(1)); // LT : 3 < 4
        assert_eq!(calcule(4, 3, 0x21), Ok(0)); // LT : 4 < 3
        assert_eq!(calcule(-1, 0, 0x21), Ok(1)); // LT : signé, -1 < 0
        assert_eq!(calcule(4, 3, 0x22), Ok(1)); // GT : 4 > 3
        assert_eq!(calcule(3, 3, 0x22), Ok(0)); // GT : 3 > 3
    }

    #[test]
    fn not_est_logique() {
        for (a, attendu) in [(0, 1), (1, 0), (5, 0), (-1, 0)] {
            let p = programme(&[0x23, 0x00]); // NOT, HALT
            let mut m = Machine::new(&p, 1000);
            m.stack.push(a).unwrap();
            assert_eq!(m.run(), Ok(()));
            assert_eq!(m.stack().clone().pop(), Ok(attendu), "NOT {a}");
        }
    }

    #[test]
    fn operations_bit_a_bit() {
        assert_eq!(calcule(6, 3, 0x24), Ok(2)); // AND : 0110 & 0011 = 0010
        assert_eq!(calcule(6, 3, 0x25), Ok(7)); // OR  : 0110 | 0011 = 0111
        assert_eq!(calcule(6, 3, 0x26), Ok(5)); // XOR : 0110 ^ 0011 = 0101
    }

    #[test]
    fn bnot_inverse_chaque_bit() {
        for (a, attendu) in [(0, -1), (-1, 0), (5, -6)] {
            let p = programme(&[0x29, 0x00]); // BNOT, HALT
            let mut m = Machine::new(&p, 1000);
            m.stack.push(a).unwrap();
            assert_eq!(m.run(), Ok(()));
            assert_eq!(m.stack().clone().pop(), Ok(attendu), "BNOT {a}");
        }
    }

    #[test]
    fn decalages() {
        assert_eq!(calcule(1, 3, 0x27), Ok(8)); // SHL : 0001 -> 1000
        assert_eq!(calcule(8, 3, 0x28), Ok(1)); // SHR : 1000 -> 0001
        assert_eq!(calcule(-1, 1, 0x28), Ok(i64::MAX)); // SHR est logique (exemple de B.6)
        assert_eq!(calcule(-1, 1, 0x2D), Ok(-1)); // SAR garde le signe (exemple de B.6)
        assert_eq!(calcule(-7, 1, 0x2D), Ok(-4)); // SAR (exemple de B.6)
    }

    #[test]
    fn decalage_hors_de_0_63_ne_faute_jamais() {
        for n in [64, 100, -1] {
            assert_eq!(calcule(5, n, 0x27), Ok(0), "SHL {n}");
            assert_eq!(calcule(-5, n, 0x28), Ok(0), "SHR {n}");
            assert_eq!(calcule(5, n, 0x2D), Ok(0), "SAR positif {n}");
            assert_eq!(calcule(-5, n, 0x2D), Ok(-1), "SAR négatif {n}");
        }
    }

    #[test]
    fn store_puis_load_a_adresse_fixe() {
        // PUSHI16 42, STORE 7, LOAD 7, HALT
        let p = programme(&[0x02, 0x00, 0x2A, 0x33, 0x07, 0x32, 0x07, 0x00]);
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Ok(()));
        assert_eq!(m.memory()[7], 42);
        assert_eq!(m.stack().clone().pop(), Ok(42));
        assert_eq!(m.gas(), 1000 - 1 - 4 - 3);
    }

    #[test]
    fn storex_puis_loadx_adresse_calculee() {
        // PUSHI16 9, PUSHI16 255, STOREX, PUSHI16 255, LOADX, HALT
        let p = programme(&[
            0x02, 0x00, 0x09, 0x02, 0x00, 0xFF, 0x31, 0x02, 0x00, 0xFF, 0x30, 0x00,
        ]);
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Ok(()));
        assert_eq!(m.memory()[255], 9);
        assert_eq!(m.stack().clone().pop(), Ok(9));
    }

    #[test]
    fn adresse_hors_memoire_donne_out_of_bounds() {
        for addr in [256, -1, i64::MAX, i64::MIN] {
            let p = programme(&[0x30, 0x00]); // LOADX, HALT
            let mut m = Machine::new(&p, 1000);
            m.stack.push(addr).unwrap();
            assert_eq!(m.run(), Err(Fault::OutOfBounds), "LOADX {addr}");

            let p = programme(&[0x31, 0x00]); // STOREX, HALT
            let mut m = Machine::new(&p, 1000);
            m.stack.push(1).unwrap();
            m.stack.push(addr).unwrap();
            assert_eq!(m.run(), Err(Fault::OutOfBounds), "STOREX {addr}");
        }
    }

    #[test]
    fn store_sur_pile_vide_donne_stack_underflow() {
        // Cas 063 du corpus : STORE 0 sur une pile vide.
        let p = programme(&[0x33, 0x00, 0x00]);
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Err(Fault::StackUnderflow));
    }

    #[test]
    fn jmp_saute_par_dessus_une_instruction() {
        // 0: PUSHI16 1 | 3: JMP +3 | 6: PUSHI16 2 | 9: JUMPDEST | 10: HALT
        // cible = 3 + 3 + 3 = 9
        let p = programme(&[
            0x02, 0x00, 0x01, 0x40, 0x00, 0x03, 0x02, 0x00, 0x02, 0x43, 0x00,
        ]);
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Ok(()));
        assert_eq!(contenu(&m), vec![1]); // PUSHI16 2 a été sauté
    }

    #[test]
    fn jnz_fait_une_boucle() {
        // Compte à rebours de 3 à 0.
        // 0: PUSHI16 3 | 3: JUMPDEST | 4: PUSHI16 1 | 7: SUB | 8: DUP | 9: JNZ -9 | 12: HALT
        // cible = 9 + 3 - 9 = 3
        let p = programme(&[
            0x02, 0x00, 0x03, 0x43, 0x02, 0x00, 0x01, 0x11, 0x04, 0x42, 0xFF, 0xF7, 0x00,
        ]);
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Ok(()));
        assert_eq!(contenu(&m), vec![0]);
        // PUSHI16 3 (1) + 3 tours de JUMPDEST, PUSHI16, SUB, DUP, JNZ (1+1+2+1+2) + HALT (0)
        assert_eq!(m.gas(), 1000 - 1 - 3 * 7);
    }

    #[test]
    fn cible_sans_jumpdest_donne_bad_jump() {
        // JMP +0 : la cible est le HALT qui suit, pas un JUMPDEST.
        let p = programme(&[0x40, 0x00, 0x00, 0x00]);
        assert_eq!(Machine::new(&p, 1000).run(), Err(Fault::BadJump));
        // JMP +100 : hors du code.
        let p = programme(&[0x40, 0x00, 0x64, 0x00]);
        assert_eq!(Machine::new(&p, 1000).run(), Err(Fault::BadJump));
        // JMP -10 : avant le début du code.
        let p = programme(&[0x40, 0xFF, 0xF6, 0x00]);
        assert_eq!(Machine::new(&p, 1000).run(), Err(Fault::BadJump));
    }

    #[test]
    fn jz_ne_verifie_la_cible_que_s_il_saute() {
        // PUSHI16 1, JZ +100, HALT : 1 n'est pas 0, donc pas de saut et pas de BadJump.
        let p = programme(&[0x02, 0x00, 0x01, 0x41, 0x00, 0x64, 0x00]);
        let mut m = Machine::new(&p, 1000);
        assert_eq!(m.run(), Ok(()));
        assert!(m.stack().is_empty()); // JZ a quand même dépilé
    }
}
