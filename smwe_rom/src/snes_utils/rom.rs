use nom::{error::Error, IResult};

use crate::{
    error::{ParseErr, RomError},
    snes_utils::{
        addr::{Addr, AddrPc},
        rom_slice::*,
    },
};

pub const SMC_HEADER_SIZE: usize = 0x200;

pub struct Rom(Vec<u8>);

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

    pub fn slice_pc(&self, slice: PcSlice) -> Result<&[u8], RomError> {
        if slice.is_infinite() {
            self.0.get(slice.begin.0..)
        } else {
            self.0.get(slice.begin.0..slice.begin.0 + slice.size)
        }
        .ok_or(RomError::SlicePc(slice))
    }

    pub fn slice_lorom(&self, slice: SnesSlice) -> Result<&[u8], RomError> {
        let begin = AddrPc::try_from_lorom(slice.begin).map_err(RomError::AddressSliceLoRom)?;
        let pc_slice = PcSlice::new(begin, slice.size);
        self.slice_pc(pc_slice).map_err(|_| RomError::SliceLoRom(slice))
    }

    pub fn slice_hirom(&self, slice: SnesSlice) -> Result<&[u8], RomError> {
        let begin = AddrPc::try_from_hirom(slice.begin).map_err(RomError::AddressSliceHiRom)?;
        let pc_slice = PcSlice::new(begin, slice.size);
        self.slice_pc(pc_slice).map_err(|_| RomError::SliceHiRom(slice))
    }

    pub fn parse_slice_pc<'r, Parser, Ret>(&'r self, slice: PcSlice, mut parser: Parser) -> Result<Ret, RomError>
    where
        Parser: FnMut(&'r [u8]) -> IResult<&'r [u8], Ret, Error<&'r [u8]>>,
    {
        let bytes = self.slice_pc(slice)?;
        let (_, ret) = parser(bytes).map_err(|_: ParseErr| RomError::ParsePc(slice))?;
        Ok(ret)
    }

    pub fn parse_slice_lorom<'r, Parser, Ret>(&'r self, slice: SnesSlice, parser: Parser) -> Result<Ret, RomError>
    where
        Parser: FnMut(&'r [u8]) -> IResult<&'r [u8], Ret, Error<&'r [u8]>>,
    {
        let begin = AddrPc::try_from_lorom(slice.begin).map_err(RomError::AddressParseLoRom)?;
        let pc_slice = PcSlice::new(begin, slice.size);
        self.parse_slice_pc(pc_slice, parser).map_err(|_| RomError::ParseLoRom(slice))
    }

    pub fn parse_slice_hirom<'r, Parser, Ret>(&'r self, slice: SnesSlice, parser: Parser) -> Result<Ret, RomError>
    where
        Parser: FnMut(&'r [u8]) -> IResult<&'r [u8], Ret, Error<&'r [u8]>>,
    {
        let begin = AddrPc::try_from_hirom(slice.begin).map_err(RomError::AddressParseHiRom)?;
        let pc_slice = PcSlice::new(begin, slice.size);
        self.parse_slice_pc(pc_slice, parser).map_err(|_| RomError::ParseHiRom(slice))
    }
}
