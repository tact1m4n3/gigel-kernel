use core::{marker::PhantomData, mem, slice, str};

use mm::PAGE_SIZE;

const MULTIBOOT_MAGIC: u64 = 0x36d76289;

pub fn init(magic: u64, info: *const u8) -> &'static MultibootInfo {
    if magic != MULTIBOOT_MAGIC {
        panic!("unknown bootloader")
    }
    unsafe { &*(info as *mut MultibootInfo) }
}

#[repr(C)]
pub struct MultibootInfo {
    size: u32,
    reserved: u32,
    tags: Tag,
}

impl MultibootInfo {
    pub fn overlap_page(&self, addr: usize) -> bool {
        let start = self as *const _ as usize;
        start < addr + PAGE_SIZE && addr <= start + (self.size as usize)
    }

    pub fn tags(&self) -> TagIter {
        TagIter {
            current: &self.tags as *const Tag,
            phantom: PhantomData {},
        }
    }

    pub fn find_tag<T: IsTag>(&self, typ: TagType) -> Option<&T> {
        unsafe { mem::transmute(self.tags().find(|&x| x.typ == typ)) }
    }
}

#[repr(C)]
pub struct Tag {
    typ: TagType,
    size: u32,
}

pub trait IsTag {}

pub struct TagIter<'a> {
    current: *const Tag,
    phantom: PhantomData<&'a Tag>,
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { &*self.current } {
            &Tag {
                typ: TagType::End,
                size: 8,
            } => None,
            tag => {
                self.current = ((self.current as usize + tag.size as usize + 7) & !7) as *const Tag;
                Some(tag)
            }
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
                self.size as usize - mem::size_of::<Tag>(),
            ))
        }
    }
}

impl IsTag for CmdlineTag {}

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
                self.size as usize - mem::size_of::<Tag>(),
            ))
        }
    }
}

impl IsTag for BootloaderTag {}

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

impl IsTag for MemoryMapTag {}

#[repr(C)]
pub struct MemoryArea {
    pub base_addr: u64,
    pub length: u64,
    pub typ: AreaType,
    reserved: u32,
}

impl MemoryArea {
    pub fn start(&self) -> usize {
        self.base_addr as usize
    }

    pub fn size(&self) -> usize {
        self.length as usize
    }

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

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AreaType {
    Available = 1,
    Reserved = 2,
    AcpiAvailable = 3,
    ReservedHibernate = 4,
    Defective = 5,
}
