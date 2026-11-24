//! Pile de valeurs de la machine (taille fixe, aucune allocation).

use pbc::Fault;

/// Nombre maximum de valeurs sur la pile (spec : 64).
pub const STACK_SIZE: usize = 64;

/// Pile de valeurs `i64` : un tableau fixe et une hauteur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
    values: [i64; STACK_SIZE],
    height: usize,
}

impl Stack {
    /// Crée une pile vide.
    pub fn new() -> Self {
        Self {
            values: [0; STACK_SIZE],
            height: 0,
        }
    }

    /// Pose une valeur au sommet. Faute si la pile est pleine.
    pub fn push(&mut self, value: i64) -> Result<(), Fault> {
        let case = self
            .values
            .get_mut(self.height)
            .ok_or(Fault::StackOverflow)?;
        *case = value;
        self.height += 1;
        Ok(())
    }

    /// Retire et renvoie la valeur du sommet. Faute si la pile est vide.
    pub fn pop(&mut self) -> Result<i64, Fault> {
        let new_height = self.height.checked_sub(1).ok_or(Fault::StackUnderflow)?;
        let value = *self.values.get(new_height).ok_or(Fault::StackUnderflow)?;
        self.height = new_height;
        Ok(value)
    }

    /// Nombre de valeurs sur la pile.
    pub fn len(&self) -> usize {
        self.height
    }

    /// Vrai si la pile est vide.
    pub fn is_empty(&self) -> bool {
        self.height == 0
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empiler_puis_depiler_rend_les_valeurs_a_l_envers() {
        let mut pile = Stack::new();
        pile.push(3).unwrap();
        pile.push(4).unwrap();
        assert_eq!(pile.len(), 2);
        assert_eq!(pile.pop(), Ok(4));
        assert_eq!(pile.pop(), Ok(3));
        assert!(pile.is_empty());
    }

    #[test]
    fn depiler_une_pile_vide_donne_stack_underflow() {
        let mut pile = Stack::new();
        assert_eq!(pile.pop(), Err(Fault::StackUnderflow));
    }

    #[test]
    fn la_65e_valeur_donne_stack_overflow() {
        let mut pile = Stack::new();
        for i in 0..64 {
            pile.push(i).unwrap();
        }
        assert_eq!(pile.push(64), Err(Fault::StackOverflow));
        assert_eq!(pile.len(), 64);
    }
}
