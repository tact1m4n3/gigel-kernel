#![no_std]

use core::{marker::PhantomData, mem, slice, str};

pub const MAGIC: u64 = 0x36d76289;

pub fn init(magic: u64, info: *const u8) -> Option<&'static BootInfo> {
    if magic == MAGIC {
        Some(unsafe { &*(info as *mut BootInfo) })
    } else {
        None
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    size: u32,
    reserved: u32,
    tags: TagBase,
}

impl BootInfo {
    #[inline]
    pub fn start_addr(&self) -> usize {
        self as *const _ as usize
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size as usize
    }

    #[inline]
    pub fn end_addr(&self) -> usize {
        self.start_addr() + self.size()
    }

    #[inline]
    pub const fn tags(&self) -> TagIter {
        TagIter {
            current: &self.tags as *const TagBase,
            phantom: PhantomData {},
        }
    }

    pub fn find_tag<T: Tag>(&self, typ: TagType) -> Option<&T> {
        unsafe { mem::transmute(self.tags().find(|&x| x.typ == typ)) }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TagBase {
    typ: TagType,
    size: u32,
}

pub trait Tag {}

pub struct TagIter<'a> {
    current: *const TagBase,
    phantom: PhantomData<&'a TagBase>,
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a TagBase;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { &*self.current } {
            &TagBase {
                typ: TagType::End,
                size: 8,
            } => None,
            tag => {
                self.current =
                    ((self.current as usize + tag.size as usize + 7) & !7) as *const TagBase;
                Some(tag)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TagType {
    End = 0,
    Cmdline = 1,
    Bootloader = 2,
    Module = 3,
    BasicMemInfo = 4,
    BootDevice = 5,
    Mmap = 6,
    VbeInfo = 7,
    Framebuffer = 8,
    ElfSections = 9,
    Apm = 10,
    Efi32 = 11,
    Efi64 = 12,
    Smbios = 13,
    AcpiV1 = 14,
    AcpiV2 = 15,
    Network = 16,
    EfiMmap = 17,
    EfiBs = 18,
    Efi32Ih = 19,
    Efi64Ih = 20,
    LoadBaseAddr = 21,
}

#[derive(Debug)]
#[repr(C)]
pub struct CmdlineTag {
    typ: TagType,
    size: u32,
    cmdline: u8,
}

impl CmdlineTag {
    pub fn get(&self) -> &'static str {
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(
                &self.cmdline,
                self.size as usize - mem::size_of::<TagBase>(),
            ))
        }
    }
}

impl Tag for CmdlineTag {}

#[derive(Debug)]
#[repr(C)]
pub struct BootloaderTag {
    typ: TagType,
    size: u32,
    bootloader: u8,
}

impl BootloaderTag {
    pub fn get(&self) -> &'static str {
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(
                &self.bootloader,
                self.size as usize - mem::size_of::<TagBase>(),
            ))
        }
    }
}

impl Tag for BootloaderTag {}

#[derive(Debug)]
#[repr(C)]
pub struct MemoryMapTag {
    typ: TagType,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    areas: MemoryArea,
}

impl MemoryMapTag {
    pub fn areas(&self) -> MemoryAreaIter {
        let start = &self.areas as *const _ as usize;
        let end = self as *const _ as usize + self.size as usize;
        MemoryAreaIter {
            current_area: start,
            last_area: end,
            entry_size: self.entry_size as usize,
            phantom: PhantomData {},
        }
    }
}

impl Tag for MemoryMapTag {}

#[derive(Debug)]
#[repr(C)]
pub struct MemoryArea {
    pub base_addr: u64,
    pub length: u64,
    pub typ: AreaType,
    reserved: u32,
}

impl MemoryArea {
    #[inline]
    pub fn start_addr(&self) -> usize {
        self.base_addr as usize
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.length as usize
    }

    #[inline]
    pub fn end_addr(&self) -> usize {
        self.start_addr() + self.size()
    }

    #[inline]
    pub fn typ(&self) -> AreaType {
        self.typ
    }
}

pub struct MemoryAreaIter<'a> {
    current_area: usize,
    last_area: usize,
    entry_size: usize,
    phantom: PhantomData<&'a MemoryArea>,
}

impl<'a> Iterator for MemoryAreaIter<'a> {
    type Item = &'a MemoryArea;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_area < self.last_area {
            let area = unsafe { &*(self.current_area as *const MemoryArea) };
            self.current_area += self.entry_size;
            Some(area)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AreaType {
    Available = 1,
    Reserved = 2,
    AcpiAvailable = 3,
    ReservedHibernate = 4,
    Defective = 5,
}
