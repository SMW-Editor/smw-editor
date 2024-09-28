use std::sync::Arc;

use thiserror::Error;

use crate::{
    compression::DecompressionError,
    disassembler::binary_block::DataBlock,
    snes_utils::{addr::AddrPc, rom_slice::*},
};

// -------------------------------------------------------------------------------------------------

type ParseErr<'a> = nom::Err<nom::error::Error<&'a [u8]>>;

#[derive(Debug, Error)]
pub enum RomError {
    #[error("Empty ROM file")]
    Empty,
    #[error("Invalid ROM size (not a multiple of 512 bytes): {0} ({0:#x})")]
    Size(usize),
    #[error("Could not PC slice ROM: {0}")]
    SlicePc(PcSlice),
    #[error("Could not SNES slice ROM: {0}")]
    SliceSnes(SnesSlice),
    #[error("Could not decompress ROM slice:\n- {0}")]
    Decompress(DecompressionError),
    #[error("Could not parse ROM slice")]
    Parse,
    #[error("Data block not found: {0:?}")]
    DataBlockNotFound(DataBlock),
}

// -------------------------------------------------------------------------------------------------

pub const SMC_HEADER_SIZE: usize = 0x200;

// -------------------------------------------------------------------------------------------------

pub trait RomView<'r> {
    fn with_error_mapper<EM, ET>(self, error_mapper: EM) -> RomViewWithErrorMapper<'r, EM, ET, Self>
    where
        EM: Fn(RomError) -> ET,
        RomViewWithErrorMapper<'r, EM, ET, Self>: Sized,
        Self: Sized,
    {
        RomViewWithErrorMapper { error_mapper, rom_view: self, _phantom: Default::default() }
    }

    fn decompress<Decompressor>(self, decompressor: Decompressor) -> Result<Decompressed, RomError>
    where
        Decompressor: 'static + Fn(&[u8]) -> Result<Vec<u8>, DecompressionError>,
        Self: Sized,
    {
        let raw_bytes = self.as_bytes()?;
        let bytes = decompressor(raw_bytes).map_err(RomError::Decompress)?;
        Ok(Decompressed { bytes })
    }

    fn parse<'s, Ret, Parser>(&'s self, mut f: Parser) -> Result<Ret, RomError>
    where
        Parser: nom::Parser<&'r [u8], Ret, nom::error::Error<&'r [u8]>>,
        Self: Sized,
    {
        let bytes = self.as_bytes()?;
        let result = f.parse(bytes);
        let (_, ret) = result.map_err(|_: ParseErr| RomError::Parse)?;
        Ok(ret)
    }

    fn as_bytes(&self) -> Result<&'r [u8], RomError>;
}

pub trait IsDecompressed {
    fn as_decompressed(&self) -> &Decompressed;
}

#[derive(Clone)]
pub struct Rom(pub Arc<[u8]>);

pub struct RomWithErrorMapper<'r, EM, ET>
where
    EM: Fn(RomError) -> ET,
{
    error_mapper: EM,
    rom:          &'r Rom,
}

pub struct RomViewWithErrorMapper<'r, EM, ET, RV>
where
    EM: Fn(RomError) -> ET,
{
    error_mapper: EM,
    rom_view:     RV,
    _phantom:     std::marker::PhantomData<&'r [u8]>,
}

#[derive(Clone)]
pub struct SnesSliced<'r> {
    pc_sliced: PcSliced<'r>,
}

#[derive(Clone)]
pub struct PcSliced<'r> {
    slice: PcSlice,
    rom:   &'r Rom,
}

pub struct Decompressed {
    bytes: Vec<u8>,
}

pub struct DecompressedView<'r> {
    decompressed: &'r Decompressed,
}

// -------------------------------------------------------------------------------------------------

pub fn noop_error_mapper<ET>(e: ET) -> ET {
    e
}

impl Rom {
    pub fn new(mut data: Vec<u8>) -> Result<Self, RomError> {
        if !data.is_empty() {
            match data.len() % 0x400 {
                SMC_HEADER_SIZE => {
                    data.drain(..SMC_HEADER_SIZE);
                    Ok(Self(Arc::from(data)))
                }
                0 => Ok(Self(Arc::from(data))),
                _ => Err(RomError::Size(data.len())),
            }
        } else {
            Err(RomError::Empty)
        }
    }

    pub fn view(&self) -> RomWithErrorMapper<'_, impl Fn(RomError) -> RomError, RomError> {
        self.with_error_mapper(noop_error_mapper)
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
    pub fn slice_pc(self, slice: PcSlice) -> RomViewWithErrorMapper<'r, EM, ET, PcSliced<'r>> {
        PcSliced { rom: self.rom, slice }.with_error_mapper(self.error_mapper)
    }

    pub fn slice_lorom(self, slice: SnesSlice) -> Result<RomViewWithErrorMapper<'r, EM, ET, SnesSliced<'r>>, ET> {
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

impl<'r, EM, ET> RomView<'r> for RomWithErrorMapper<'r, EM, ET>
where
    EM: Fn(RomError) -> ET,
{
    fn as_bytes(&self) -> Result<&'r [u8], RomError> {
        Ok(&self.rom.0)
    }
}

impl<'r, EM, ET, RV: RomView<'r>> RomViewWithErrorMapper<'r, EM, ET, RV>
where
    EM: Fn(RomError) -> ET,
    RV: RomView<'r>,
    Self: Sized,
{
    pub fn decompress<Decompressor>(
        self, decompressor: Decompressor,
    ) -> Result<RomViewWithErrorMapper<'r, EM, ET, Decompressed>, ET>
    where
        Self: Sized,
        Decompressor: 'static + Fn(&[u8]) -> Result<Vec<u8>, DecompressionError>,
    {
        let decomp = self.rom_view.decompress(decompressor).map_err(&self.error_mapper)?;
        Ok(RomViewWithErrorMapper {
            error_mapper: self.error_mapper,
            rom_view:     decomp,
            _phantom:     Default::default(),
        })
    }

    pub fn parse<'s, Ret: 's, Parser>(&'s self, f: Parser) -> Result<Ret, ET>
    where
        Parser: nom::Parser<&'r [u8], Ret, nom::error::Error<&'r [u8]>>,
        Self: Sized,
    {
        self.rom_view.parse(f).map_err(&self.error_mapper)
    }

    pub fn as_bytes(&self) -> Result<&'r [u8], ET> {
        self.rom_view.as_bytes().map_err(&self.error_mapper)
    }
}

impl<'r, EM, ET, RV> RomViewWithErrorMapper<'r, EM, ET, RV>
where
    EM: Fn(RomError) -> ET,
    RV: IsDecompressed,
    Self: Sized,
{
    pub fn view(&'r self) -> RomViewWithErrorMapper<'r, &'r EM, ET, DecompressedView<'r>> {
        self.rom_view.as_decompressed().view().with_error_mapper(&self.error_mapper)
    }
}

impl<'r> RomView<'r> for SnesSliced<'r> {
    fn as_bytes(&self) -> Result<&'r [u8], RomError> {
        self.pc_sliced.as_bytes()
    }
}

impl<'r> RomView<'r> for PcSliced<'r> {
    fn as_bytes(&self) -> Result<&'r [u8], RomError> {
        let PcSliced { rom, slice } = self;
        if slice.is_infinite() {
            rom.0.get(slice.begin.as_index()..)
        } else {
            rom.0.get(slice.begin.as_index()..slice.begin.as_index() + slice.size)
        }
        .ok_or(RomError::SlicePc(*slice))
    }
}

impl<'r> RomView<'r> for DecompressedView<'r> {
    fn as_bytes(&self) -> Result<&'r [u8], RomError> {
        Ok(&self.decompressed.bytes)
    }
}

impl IsDecompressed for Decompressed {
    fn as_decompressed(&self) -> &Decompressed {
        self
    }
}

impl Decompressed {
    pub fn view(&self) -> DecompressedView {
        DecompressedView { decompressed: self }
    }
}
