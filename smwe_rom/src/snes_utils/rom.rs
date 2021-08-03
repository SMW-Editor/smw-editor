use crate::{
    error::{DecompressionError, ParseErr, RomError},
    snes_utils::{
        addr::{Addr, AddrPc},
        rom_slice::*,
    },
};

pub const SMC_HEADER_SIZE: usize = 0x200;

pub trait RomView {
    fn with_error_mapper<EM, ET>(self, error_mapper: EM) -> RomViewWithErrorMapper<EM, ET, Self>
    where
        EM: Fn(RomError) -> ET,
        RomViewWithErrorMapper<EM, ET, Self>: Sized,
        Self: Sized,
    {
        RomViewWithErrorMapper { error_mapper, rom_view: self }
    }

    fn decompress<Decompressor>(self, decompressor: Decompressor) -> Result<Decompressed, RomError>
    where
        Self: Sized,
        Decompressor: 'static + Fn(&[u8]) -> Result<Vec<u8>, DecompressionError>,
    {
        let raw_bytes = self.as_bytes()?;
        let bytes = decompressor(raw_bytes).map_err(RomError::Decompress)?;
        Ok(Decompressed { bytes })
    }

    fn parse<'s, Ret, Parser>(&'s mut self, mut f: Parser) -> Result<Ret, RomError>
    where
        Parser: nom::Parser<&'s [u8], Ret, nom::error::Error<&'s [u8]>>,
        Self: Sized,
    {
        let bytes = self.as_bytes()?;
        let (_, ret) = f.parse(bytes).map_err(|_: ParseErr| RomError::Parse)?;
        Ok(ret)
    }

    fn as_bytes(&self) -> Result<&[u8], RomError>;

    fn into_bytes(self) -> Result<Vec<u8>, RomError>
    where
        Self: Sized,
    {
        Ok(self.as_bytes()?.to_owned())
    }
}

pub struct Rom(Vec<u8>);

pub struct RomWithErrorMapper<'r, EM, ET>
where
    EM: Fn(RomError) -> ET,
{
    error_mapper: EM,
    rom:          &'r Rom,
}

pub struct RomViewWithErrorMapper<EM, ET, RV: RomView>
where
    EM: Fn(RomError) -> ET,
{
    error_mapper: EM,
    rom_view:     RV,
}

pub struct SnesSliced<'r> {
    pc_sliced: PcSliced<'r>,
}

pub struct PcSliced<'r> {
    slice: PcSlice,
    rom:   &'r Rom,
}

pub struct Decompressed {
    bytes: Vec<u8>,
}

impl Rom {
    pub fn new(mut data: Vec<u8>) -> Result<Self, RomError> {
        if !data.is_empty() {
            let modulo_1k = data.len() % 0x400;
            if modulo_1k == 0 {
                Ok(Self(data))
            } else if modulo_1k == SMC_HEADER_SIZE {
                Ok(Self(data.drain(SMC_HEADER_SIZE..).collect()))
            } else {
                Err(RomError::Size(data.len()))
            }
        } else {
            Err(RomError::Empty)
        }
    }

    pub fn view(&self) -> RomWithErrorMapper<'_, impl Fn(RomError) -> RomError, RomError> {
        self.with_error_mapper(|e| e)
    }

    pub fn with_error_mapper<'r, EM, ET>(&'r self, error_mapper: EM) -> RomWithErrorMapper<'r, EM, ET>
    where
        EM: Fn(RomError) -> ET,
        RomWithErrorMapper<'r, EM, ET>: Sized,
        Self: Sized,
    {
        RomWithErrorMapper { error_mapper, rom: self }
    }
}

impl<'r, EM, ET> RomWithErrorMapper<'r, EM, ET>
where
    EM: Fn(RomError) -> ET,
{
    pub fn slice_pc(self, slice: PcSlice) -> RomViewWithErrorMapper<EM, ET, PcSliced<'r>> {
        PcSliced { rom: self.rom, slice }.with_error_mapper(self.error_mapper)
    }

    pub fn slice_lorom(self, slice: SnesSlice) -> Result<RomViewWithErrorMapper<EM, ET, SnesSliced<'r>>, ET> {
        Ok(SnesSliced {
            pc_sliced: {
                let begin = AddrPc::try_from_lorom(slice.begin)
                    .map_err(|_| RomError::SliceSnes(slice))
                    .map_err(&self.error_mapper)?;
                let slice = PcSlice::new(begin, slice.size);
                PcSliced { rom: self.rom, slice }
            },
        }
        .with_error_mapper(self.error_mapper))
    }
}

impl<'r, EM, ET> RomView for RomWithErrorMapper<'r, EM, ET>
where
    EM: Fn(RomError) -> ET,
{
    fn as_bytes(&self) -> Result<&[u8], RomError> {
        Ok(&self.rom.0)
    }
}

impl<EM, ET, RV: RomView> RomViewWithErrorMapper<EM, ET, RV>
where
    EM: Fn(RomError) -> ET,
    Self: Sized,
{
    pub fn decompress<Decompressor>(
        self, decompressor: Decompressor,
    ) -> Result<RomViewWithErrorMapper<EM, ET, Decompressed>, ET>
    where
        Self: Sized,
        Decompressor: 'static + Fn(&[u8]) -> Result<Vec<u8>, DecompressionError>,
    {
        Ok(self.rom_view.decompress(decompressor).map_err(&self.error_mapper)?.with_error_mapper(self.error_mapper))
    }

    pub fn parse<'s, Ret, Parser>(&'s mut self, f: Parser) -> Result<Ret, ET>
    where
        Parser: nom::Parser<&'s [u8], Ret, nom::error::Error<&'s [u8]>>,
        Self: Sized,
    {
        self.rom_view.parse(f).map_err(&self.error_mapper)
    }

    pub fn as_bytes(&self) -> Result<&[u8], ET> {
        self.rom_view.as_bytes().map_err(&self.error_mapper)
    }

    pub fn into_bytes(self) -> Result<Vec<u8>, ET>
    where
        Self: Sized,
    {
        self.rom_view.into_bytes().map_err(&self.error_mapper)
    }
}

impl<'r> RomView for SnesSliced<'r> {
    fn as_bytes(&self) -> Result<&[u8], RomError> {
        self.pc_sliced.as_bytes()
    }
}

impl<'r> RomView for PcSliced<'r> {
    fn as_bytes(&self) -> Result<&[u8], RomError> {
        let PcSliced { rom, slice } = self;
        if slice.is_infinite() {
            rom.0.get(slice.begin.0..)
        } else {
            rom.0.get(slice.begin.0..slice.begin.0 + slice.size)
        }
        .ok_or(RomError::SlicePc(*slice))
    }
}

impl RomView for Decompressed {
    fn as_bytes(&self) -> Result<&[u8], RomError> {
        Ok(&self.bytes)
    }

    fn into_bytes(self) -> Result<Vec<u8>, RomError> {
        Ok(self.bytes)
    }
}
