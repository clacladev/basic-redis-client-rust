const OP_CODE_EOF: u8 = 0xff;
const OP_CODE_SELECTDB: u8 = 0xfe;
const OP_CODE_EXPIRETIME_S: u8 = 0xfd;
const OP_CODE_EXPIRETIME_MS: u8 = 0xfc;
const OP_CODE_RESIZEDB: u8 = 0xfb;
const OP_CODE_AUX: u8 = 0xfa;

#[derive(Debug)]
pub enum OpCode {
    EOF,
    SelectDB,
    ExpireTimeS,
    ExpireTimeMS,
    ResizeDB,
    Aux,
}

impl TryFrom<u8> for OpCode {
    type Error = anyhow::Error;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            OP_CODE_EOF => Ok(Self::EOF),
            OP_CODE_SELECTDB => Ok(Self::SelectDB),
            OP_CODE_EXPIRETIME_S => Ok(Self::ExpireTimeS),
            OP_CODE_EXPIRETIME_MS => Ok(Self::ExpireTimeMS),
            OP_CODE_RESIZEDB => Ok(Self::ResizeDB),
            OP_CODE_AUX => Ok(Self::Aux),
            _ => anyhow::bail!("Unknown op code: {:#02X}", val),
        }
    }
}
