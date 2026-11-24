//! Les instructions PBC et leur décodage (annexe B.3).

use crate::Fault;
use crate::reader::Reader;

/// Une instruction décodée. Celles qui ont un opérande le portent avec elles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// `0x00` — Fin normale du tour.
    Halt,
    /// `0x01` — Empile `constants[k]`.
    Const(u16),
    /// `0x02` — Empile `n`, étendu en signe.
    PushI16(i16),
    /// `0x03` — Dépile et jette le sommet.
    Drop,
    /// `0x04` — Duplique le sommet.
    Dup,
    /// `0x05` — Échange les deux valeurs du sommet.
    Swap,
    /// `0x06` — Copie la deuxième valeur au sommet.
    Over,
    /// `0x07` — Rotation des trois valeurs du sommet.
    Rot,
    /// `0x08` — Empile `constants[k]`, `k` dépilé.
    ConstX,
    /// `0x10` — Addition enveloppante.
    Add,
    /// `0x11` — Soustraction enveloppante.
    Sub,
    /// `0x12` — Multiplication enveloppante.
    Mul,
    /// `0x13` — Division entière.
    Div,
    /// `0x14` — Reste de la division.
    Mod,
    /// `0x15` — Opposé.
    Neg,
    /// `0x20` — Égalité : `1` ou `0`.
    Eq,
    /// `0x21` — Inférieur strict, signé.
    Lt,
    /// `0x22` — Supérieur strict, signé.
    Gt,
    /// `0x23` — Non logique : `1` si zéro.
    Not,
    /// `0x24` — Et bit à bit.
    And,
    /// `0x25` — Ou bit à bit.
    Or,
    /// `0x26` — Ou exclusif bit à bit.
    Xor,
    /// `0x27` — Décalage logique à gauche.
    Shl,
    /// `0x28` — Décalage logique à droite.
    Shr,
    /// `0x29` — Complément à un, bit à bit.
    BNot,
    /// `0x2D` — Décalage arithmétique à droite.
    Sar,
    /// `0x30` — Lit `mem[addr]`, `addr` dépilé.
    LoadX,
    /// `0x31` — Écrit `mem[addr]`, `addr` dépilé.
    StoreX,
    /// `0x32` — Lit `mem[k]`.
    Load(u8),
    /// `0x33` — Écrit `mem[k]`.
    Store(u8),
    /// `0x40` — Saut relatif.
    Jmp(i16),
    /// `0x41` — Saut relatif si zéro.
    Jz(i16),
    /// `0x42` — Saut relatif si non nul.
    Jnz(i16),
    /// `0x43` — Marque une cible de saut.
    JumpDest,
    /// `0x44` — Saut à la cible absolue dépilée.
    JmpX,
    /// `0x50` — Appel relatif.
    Call(i16),
    /// `0x51` — Retour d'appel.
    Return,
    /// `0x52` — Appel à la cible absolue dépilée.
    CallX,
    /// `0x60` — Lit le capteur `k`.
    Sense(u8),
    /// `0x61` — Tente l'action `k`.
    Act(u8),
    /// `0x70` — Tire une valeur aléatoire.
    Rand,
    /// `0x71` — Empile le gaz restant.
    Gas,
    /// `0xF0` — Ajoute le sommet à la trace.
    Trace,
}

impl Instruction {
    /// Taille de l'instruction en octets : l'opcode plus son opérande.
    pub fn size(&self) -> usize {
        match self {
            Instruction::Load(_)
            | Instruction::Store(_)
            | Instruction::Sense(_)
            | Instruction::Act(_) => 2,
            Instruction::Const(_)
            | Instruction::PushI16(_)
            | Instruction::Jmp(_)
            | Instruction::Jz(_)
            | Instruction::Jnz(_)
            | Instruction::Call(_) => 3,
            _ => 1,
        }
    }
}

/// Décode l'instruction qui commence à la position `pc` du code (B.2, étapes 1 à 3).
///
/// Seul l'octet à `pc` et ses opérandes sont lus : rien de ce qui précède (A.3).
pub fn decode(code: &[u8], pc: usize) -> Result<Instruction, Fault> {
    let opcode = *code.get(pc).ok_or(Fault::PcOutOfRange)?;
    let operandes = code.get(pc.saturating_add(1)..).unwrap_or_default();
    let mut lecteur = Reader::new(operandes);

    let instruction = match opcode {
        0x00 => Instruction::Halt,
        0x01 => Instruction::Const(lecteur.read_u16().ok_or(Fault::Truncated)?),
        0x02 => Instruction::PushI16(lecteur.read_i16().ok_or(Fault::Truncated)?),
        0x03 => Instruction::Drop,
        0x04 => Instruction::Dup,
        0x05 => Instruction::Swap,
        0x06 => Instruction::Over,
        0x07 => Instruction::Rot,
        0x08 => Instruction::ConstX,
        0x10 => Instruction::Add,
        0x11 => Instruction::Sub,
        0x12 => Instruction::Mul,
        0x13 => Instruction::Div,
        0x14 => Instruction::Mod,
        0x15 => Instruction::Neg,
        0x20 => Instruction::Eq,
        0x21 => Instruction::Lt,
        0x22 => Instruction::Gt,
        0x23 => Instruction::Not,
        0x24 => Instruction::And,
        0x25 => Instruction::Or,
        0x26 => Instruction::Xor,
        0x27 => Instruction::Shl,
        0x28 => Instruction::Shr,
        0x29 => Instruction::BNot,
        0x2D => Instruction::Sar,
        0x30 => Instruction::LoadX,
        0x31 => Instruction::StoreX,
        0x32 => Instruction::Load(lecteur.read_u8().ok_or(Fault::Truncated)?),
        0x33 => Instruction::Store(lecteur.read_u8().ok_or(Fault::Truncated)?),
        0x40 => Instruction::Jmp(lecteur.read_i16().ok_or(Fault::Truncated)?),
        0x41 => Instruction::Jz(lecteur.read_i16().ok_or(Fault::Truncated)?),
        0x42 => Instruction::Jnz(lecteur.read_i16().ok_or(Fault::Truncated)?),
        0x43 => Instruction::JumpDest,
        0x44 => Instruction::JmpX,
        0x50 => Instruction::Call(lecteur.read_i16().ok_or(Fault::Truncated)?),
        0x51 => Instruction::Return,
        0x52 => Instruction::CallX,
        0x60 => Instruction::Sense(lecteur.read_u8().ok_or(Fault::Truncated)?),
        0x61 => Instruction::Act(lecteur.read_u8().ok_or(Fault::Truncated)?),
        0x70 => Instruction::Rand,
        0x71 => Instruction::Gas,
        0xF0 => Instruction::Trace,
        _ => return Err(Fault::BadOpcode),
    };
    Ok(instruction)
}

#[cfg(test)]
mod tests {
    use super::{Instruction, decode};
    use crate::Fault;

    /// Le code du programme `209-call-a-subroutine-twice` du corpus.
    const CARRES: [u8; 19] = [
        0x02, 0x00, 0x03, 0x50, 0x00, 0x09, 0xf0, 0x02, 0x00, 0x0c, 0x50, 0x00, 0x02, 0xf0, 0x00,
        0x43, 0x04, 0x12, 0x51,
    ];

    #[test]
    fn decode_tout_le_programme_209() {
        let attendu = [
            Instruction::PushI16(3),
            Instruction::Call(9),
            Instruction::Trace,
            Instruction::PushI16(12),
            Instruction::Call(2),
            Instruction::Trace,
            Instruction::Halt,
            Instruction::JumpDest,
            Instruction::Dup,
            Instruction::Mul,
            Instruction::Return,
        ];
        let mut pc = 0;
        for instruction in attendu {
            let decodee = decode(&CARRES, pc).unwrap();
            assert_eq!(decodee, instruction);
            pc += decodee.size();
        }
        assert_eq!(pc, CARRES.len());
    }

    #[test]
    fn lit_un_operande_negatif() {
        assert_eq!(decode(&[0x02, 0xff, 0xfe], 0), Ok(Instruction::PushI16(-2)));
    }

    #[test]
    fn pc_hors_du_code() {
        assert_eq!(decode(&[0x00], 1), Err(Fault::PcOutOfRange));
    }

    #[test]
    fn opcode_inconnu() {
        assert_eq!(decode(&[0x09], 0), Err(Fault::BadOpcode));
    }

    #[test]
    fn operande_coupe_par_la_fin_du_code() {
        assert_eq!(decode(&[0x02, 0x00], 0), Err(Fault::Truncated));
        assert_eq!(decode(&[0x32], 0), Err(Fault::Truncated));
    }

    #[test]
    fn opcode_inconnu_avant_operande_tronque() {
        // Un octet inconnu en fin de code est BadOpcode, jamais Truncated (B.2 : étape 2 avant 3).
        assert_eq!(decode(&[0x00, 0xff], 1), Err(Fault::BadOpcode));
    }

    #[test]
    fn chaque_octet_est_decode_ou_refuse_sans_paniquer() {
        let mut connus = 0;
        for octet in 0..=255u8 {
            match decode(&[octet, 0x00, 0x00], 0) {
                Ok(instruction) => {
                    connus += 1;
                    assert!(instruction.size() <= 3);
                }
                Err(faute) => assert_eq!(faute, Fault::BadOpcode),
            }
        }
        assert_eq!(connus, 43);
    }
}
