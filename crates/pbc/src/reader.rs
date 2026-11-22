//! Lecture d'octets dans l'ordre, sans jamais sortir du tableau.

/// Un curseur sur une suite d'octets : chaque lecture consomme les octets lus.
#[derive(Debug)]
pub(crate) struct Reader<'a> {
    /// Les octets qui restent à lire.
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    /// Crée un lecteur placé au début de `bytes`.
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Reader { rest: bytes }
    }

    /// Lit les `n` prochains octets, ou `None` s'il en reste moins de `n`.
    pub(crate) fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let (lus, reste) = self.rest.split_at_checked(n)?;
        self.rest = reste;
        Some(lus)
    }

    /// Lit un octet.
    pub(crate) fn read_u8(&mut self) -> Option<u8> {
        let octets = self.take(1)?;
        octets.first().copied()
    }

    /// Lit un entier non signé sur 2 octets, gros-boutiste.
    pub(crate) fn read_u16(&mut self) -> Option<u16> {
        let octets: [u8; 2] = self.take(2)?.try_into().ok()?;
        Some(u16::from_be_bytes(octets))
    }

    /// Lit un entier non signé sur 4 octets, gros-boutiste.
    pub(crate) fn read_u32(&mut self) -> Option<u32> {
        let octets: [u8; 4] = self.take(4)?.try_into().ok()?;
        Some(u32::from_be_bytes(octets))
    }

    /// Lit un entier signé sur 8 octets, gros-boutiste.
    pub(crate) fn read_i64(&mut self) -> Option<i64> {
        let octets: [u8; 8] = self.take(8)?.try_into().ok()?;
        Some(i64::from_be_bytes(octets))
    }

    /// Vrai s'il ne reste plus aucun octet à lire.
    pub(crate) fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn lit_dans_l_ordre_en_gros_boutiste() {
        let mut lecteur = Reader::new(&[0x01, 0x00, 0x13, 0x00, 0x00, 0x01, 0x00]);
        assert_eq!(lecteur.read_u8(), Some(1));
        assert_eq!(lecteur.read_u16(), Some(0x0013));
        assert_eq!(lecteur.read_u32(), Some(256));
        assert!(lecteur.is_empty());
    }

    #[test]
    fn rend_none_quand_il_manque_des_octets() {
        let mut lecteur = Reader::new(&[0x00, 0x00, 0x00]);
        assert_eq!(lecteur.read_u32(), None);
    }

    #[test]
    fn lit_un_i64_negatif() {
        let mut lecteur = Reader::new(&[0xff; 8]);
        assert_eq!(lecteur.read_i64(), Some(-1));
    }
}
