
struct FVT {
	char tags[1];
	u8 tag;
	if (tag == 0x45) { // E: Lighting Stage "DEND_FVT"
		char tag[6];
		le u32 u32_unknown0;
		u8 u8_unknown0;
		u8 text_length;
		u8 u8_unknown1;
	    char text[text_length];
	} else if (tag == 0x32) { // 2: Burning Stage "D2_FVT"
		char tag[4];
		le u32 u32_unknown0;
		le u32 u32_unknown1;
		le u32 u32_unknown2;
		u8 u8_unknown0;
		u8 text_length;
		u8 u8_unknown1;
	    char text[text_length];
	} else if (tag == 0x33) { // 3: Climax Stage & Rising Stage "D3_FVT"
		char tag[4];
		le u32 u32_unknown0;
		le u32 u32_unknown1;
		le u32 u32_unknown2;
		u8 u8_unknown0;
		u8 text_length;
		u8 u8_unknown1;
	    char text[text_length];
	}
};

FVT fvt @ 0;