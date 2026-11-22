//! Le programme chargé depuis un fichier `.pbc` (annexe A).

use crate::Fault;
use crate::reader::Reader;

/// La signature d'un fichier PBC : les lettres `P`, `B`, `C`, `1`.
const MAGIC: &[u8] = b"PBC1";
/// La seule version du format acceptée.
const VERSION: u8 = 0x01;
/// La seule valeur acceptée pour l'octet `flags` en v1.
const FLAGS: u8 = 0x00;
/// Nombre maximal de constantes (A.1).
const MAX_CONST_COUNT: u16 = 4095;
/// Taille maximale du code, en octets (A.1).
const MAX_CODE_LEN: u32 = 65_535;

/// Un programme PBC valide : sa table de constantes et son code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    constants: Vec<i64>,
    code: Vec<u8>,
}

impl Program {
    /// Charge un programme depuis les octets d'un fichier `.pbc`.
    ///
    /// Toute anomalie produit [`Fault::BadHeader`] (A.2). Ne panique jamais, quelle que soit
    /// l'entrée.
    pub fn load(bytes: &[u8]) -> Result<Program, Fault> {
        Self::parse(bytes).ok_or(Fault::BadHeader)
    }

    /// Lit l'en-tête puis le contenu, dans l'ordre de A.1. `None` dès qu'une règle est violée.
    fn parse(bytes: &[u8]) -> Option<Program> {
        let mut reader = Reader::new(bytes);

        if reader.take(4)? != MAGIC {
            return None;
        }
        if reader.read_u8()? != VERSION {
            return None;
        }
        if reader.read_u8()? != FLAGS {
            return None;
        }

        let const_count = reader.read_u16()?;
        if const_count > MAX_CONST_COUNT {
            return None;
        }
        let mut constants = Vec::with_capacity(usize::from(const_count));
        for _ in 0..const_count {
            constants.push(reader.read_i64()?);
        }

        let code_len = reader.read_u32()?;
        if code_len == 0 || code_len > MAX_CODE_LEN {
            return None;
        }
        let code = reader.take(usize::try_from(code_len).ok()?)?.to_vec();

        if !reader.is_empty() {
            return None;
        }

        Some(Program { constants, code })
    }

    /// La table des constantes.
    pub fn constants(&self) -> &[i64] {
        &self.constants
    }

    /// Les octets du code.
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

#[cfg(test)]
mod tests {
    use super::Program;
    use crate::Fault;

    /// Un en-tête valide sans constante, suivi de `code_len = 1` et d'un octet de code (`HALT`).
    const MINIMAL: [u8; 13] = [
        0x50, 0x42, 0x43, 0x31, // magic "PBC1"
        0x01, // version
        0x00, // flags
        0x00, 0x00, // const_count = 0
        0x00, 0x00, 0x00, 0x01, // code_len = 1
        0x00, // code : HALT
    ];

    #[test]
    fn charge_un_programme_minimal() {
        let programme = Program::load(&MINIMAL).unwrap();
        assert_eq!(programme.constants(), &[] as &[i64]);
        assert_eq!(programme.code(), &[0x00]);
    }

    #[test]
    fn lit_les_constantes() {
        let octets = [
            0x50, 0x42, 0x43, 0x31, 0x01, 0x00, // magic, version, flags
            0x00, 0x02, // const_count = 2
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2a, // 42
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // -1
            0x00, 0x00, 0x00, 0x01, 0x00, // code_len = 1, HALT
        ];
        let programme = Program::load(&octets).unwrap();
        assert_eq!(programme.constants(), &[42, -1]);
    }

    #[test]
    fn refuse_une_mauvaise_signature() {
        let mut octets = MINIMAL;
        octets[3] = b'2';
        assert_eq!(Program::load(&octets), Err(Fault::BadHeader));
    }

    #[test]
    fn refuse_une_mauvaise_version_ou_des_flags() {
        let mut version = MINIMAL;
        version[4] = 0x02;
        assert_eq!(Program::load(&version), Err(Fault::BadHeader));

        let mut flags = MINIMAL;
        flags[5] = 0x01;
        assert_eq!(Program::load(&flags), Err(Fault::BadHeader));
    }

    #[test]
    fn refuse_un_octet_en_trop() {
        let mut octets = MINIMAL.to_vec();
        octets.push(0x00);
        assert_eq!(Program::load(&octets), Err(Fault::BadHeader));
    }

    #[test]
    fn refuse_un_code_vide() {
        let mut octets = MINIMAL;
        octets[11] = 0x00; // code_len = 0
        assert_eq!(Program::load(&octets[..12]), Err(Fault::BadHeader));
    }

    #[test]
    fn refuse_trop_de_constantes_sans_rien_allouer() {
        let mut octets = MINIMAL;
        octets[6] = 0x10; // const_count = 4096
        assert_eq!(Program::load(&octets), Err(Fault::BadHeader));
    }

    #[test]
    fn refuse_tout_fichier_tronque() {
        for longueur in 0..MINIMAL.len() {
            assert_eq!(Program::load(&MINIMAL[..longueur]), Err(Fault::BadHeader));
        }
    }
}
