//! Les seize fautes de la spécification (annexe B.5).

/// Une faute de la machine PBC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Fault {
    #[error("BadHeader")]
    BadHeader,
    #[error("BadConst")]
    BadConst,
    #[error("BadOpcode")]
    BadOpcode,
    #[error("Truncated")]
    Truncated,
    #[error("PcOutOfRange")]
    PcOutOfRange,
    #[error("BadJump")]
    BadJump,
    #[error("StackUnderflow")]
    StackUnderflow,
    #[error("StackOverflow")]
    StackOverflow,
    #[error("CallStackOverflow")]
    CallStackOverflow,
    #[error("CallStackUnderflow")]
    CallStackUnderflow,
    #[error("OutOfGas")]
    OutOfGas,
    #[error("DivByZero")]
    DivByZero,
    #[error("OutOfBounds")]
    OutOfBounds,
    #[error("BadSensor")]
    BadSensor,
    #[error("BadAction")]
    BadAction,
    #[error("AlreadyActed")]
    AlreadyActed,
}

#[cfg(test)]
mod tests {
    use super::Fault;

    #[test]
    fn le_texte_affiche_est_celui_de_la_spec() {
        let attendus = [
            (Fault::BadHeader, "BadHeader"),
            (Fault::BadConst, "BadConst"),
            (Fault::BadOpcode, "BadOpcode"),
            (Fault::Truncated, "Truncated"),
            (Fault::PcOutOfRange, "PcOutOfRange"),
            (Fault::BadJump, "BadJump"),
            (Fault::StackUnderflow, "StackUnderflow"),
            (Fault::StackOverflow, "StackOverflow"),
            (Fault::CallStackOverflow, "CallStackOverflow"),
            (Fault::CallStackUnderflow, "CallStackUnderflow"),
            (Fault::OutOfGas, "OutOfGas"),
            (Fault::DivByZero, "DivByZero"),
            (Fault::OutOfBounds, "OutOfBounds"),
            (Fault::BadSensor, "BadSensor"),
            (Fault::BadAction, "BadAction"),
            (Fault::AlreadyActed, "AlreadyActed"),
        ];
        for (faute, texte) in attendus {
            assert_eq!(faute.to_string(), texte);
        }
    }
}