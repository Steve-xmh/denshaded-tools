struct Entry {
    char name[64];
    le u32 name_crc32;
    le u32 unknown1; // Unknown Data, it's important.
    le u32 offset;
    le u32 size;
    le u32 encrypted;
};

struct File {
    char sign[4];
    le u32 count;
    Entry entries[count];
};

File file @ 0x0;