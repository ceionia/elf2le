[MAP ALL]
[SECTION MZ start=0x0000]
dw 'MZ'     ; 00 magic num
dw 0x0000   ; 02 bytes in last block
dw 0x0006   ; 04 blocks in EXE
dw 0x0000   ; 06 relocations
dw 0x0004   ; 08 paragraphs in header
dw 0x0000   ; 0A paragraphs of additional memory (BSS)
dw 0xffff   ; 0C max extra mem
dw 0x0000   ; 0E stack segment relative value
dw 0x0500   ; 10 inital value of SP
dw 0x0000   ; 12 checksum (0)
dw 0x0000   ; 14 inital value of IP
dw 0x0000   ; 16 inital value of CS
dw 0x0040   ; 18 offset of first relocation (0x40 for LE)
dw 0x0000   ; 1A overlay num, 0 for main program

TIMES 0x3C - ($ - $$) db 0
dd 0x00000080 ; 3C LE header offset

[SECTION MSDOS start=0x0040 vstart=0x0000]
; MS-DOS Stub
[BITS 16]
push cs
pop ds
mov dx,stubtext
mov ah,0x9
int 0x21
mov ax,0x4c01
int 0x21
stubtext: db `This program cannot be run in DOS mode. (idiot)\r\n$`

[SECTION LE start=0x0080]
; LE header
LE_HEADER:
dw 'LE'       ; 00 LE signature
db 0x00       ; 02 Byte order (little-endian)
db 0x00       ; 03 Word order (little-endian)
dd 0x00000000 ; 04 executable format level
dw 0x0002     ; 08 CPU type (i386)
dw 0x0001     ; 0A Target OS (OS/2)
dd 0x00000000 ; 0C Module version
dd 0x00000200 ; 10 Module type flags
dd 0x00000002 ; 14 Number of memory pages
dd 0x00000001 ; 18 Inital object CS number
dd 0x00000000 ; 1C Inital EIP
dd 0x00000002 ; 20 Inital object SS number
dd 0x00080008 ; 24 Inital ESP
dd 0x00001000 ; 28 Memory page size
dd 0x00001000 ; 2C Bytes on last page
dd 0x00000100 ; 30 Fix-up section size
dd 0x00000000 ; 34 Fix-up section checksum
dd 0x00000000 ; 38 Loader section size
dd 0x00000000 ; 3C Loader section checksum
dd 0x000000c4 ; 40 Offset of object table
dd 0x00000002 ; 44 Object table entries
dd 0x000000f4 ; 48 Object page map offset
dd 0x00000000 ; 4C Object iterate data map offset
dd 0x000000fc ; 50 Resource table offset
dd 0x00000000 ; 54 Resource table entries
dd 0x000000fc ; 58 Resident name tables offset
dd 0x00000105 ; 5C Entry table offset
dd 0x00000000 ; 60 Module directives table offset
dd 0x00000000 ; 64 Module directives entry
dd 0x00000106 ; 68 Fix-up page table offset
dd 0x00000112 ; 6C Fix-up record table offset
dd 0x00000000 ; 70 Imported modules name table offset
dd 0x00000000 ; 74 Imported modules count
dd 0x00000000 ; 78 Imported procedure name table offset
dd 0x00000000 ; 7C Per-page checksum table offset
dd 0x00001000 ; 80 Data pages offset from top of file
dd 0x00000000 ; 84 Preload page count
dd 0x00000000 ; 88 Non-resident names table offset from top of file
dd 0x00000000 ; 8C Non-resident names table length
dd 0x00000000 ; 90 Non-resident names table checksum
dd 0x00000000 ; 94 Automatic data object
dd 0x00000000 ; 98 Debug information offset
dd 0x00000000 ; 9C Debug information length
dd 0x00000000 ; A0 Preload instance pages number
dd 0x00000000 ; A4 Demand instance pages number
dd 0x00000000 ; A8 Extra heap allocation
dd 0x00000000 ; AC ???
TIMES 0xC4 - ($ - LE_HEADER) db 0
; Object Table
; CS @ C4h
dd 0x00080000 ; 00 Virtual segment size
dd 0x00000000 ; 04 Relocation base address
dd 0x00002045 ; 08 Object flags
dd 0x00000001 ; 0C Page map index
dd 0x00000001 ; 10 Page map entries
dd 0x00000000 ; 14 ???
; SS @ DCh
dd 0x00080080 ; 00 Virtual segment size
dd 0x00070000 ; 04 Relocation base address
dd 0x00002043 ; 08 Object flags
dd 0x00000002 ; 0C Page map index
dd 0x00000001 ; 10 Page map entries
dd 0x00000000 ; 14 ???
; Page Map @ F4
; 3 byte Page Number, 1 byte Flags
; 1, Code
db 0x00, 0x00, 0x01  ; 3 byte Page Number
db 0x00              ; Flags
db 0x00, 0x00, 0x02  ; 3 byte Page Number
db 0x00              ; Flags
; Resource Table (0) & Resident Name Table
db 5 ; String Length
db 'hello' ; ASCII string
dw 0x0000 ; Ordinal Number
db 0x00 ; ???
; 105h Entry Table?
db 0x00
; 106h Fixup Page Table
dd 0x00000000 ; Record Offset of Page 1
dd 0x00000007 ; Record Offset of Page 2
dd 0x0000000E ; End of Records
; 112h Fixup Record Table
fixup_table_start:
db 0x07, 0x00 ; 32-bit internal relocation
dw 0x0000 ; source offset in page
db 0x01 ; target object
dw 0x0000 ; text - entry ; target offset
db 0x07, 0x00 ; 32-bit internal relocation
dw 0x0000 ; source offset in page
db 0x02 ; target object
dw 0x0000 ; target offset
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
db 0x07, 0x00, 0xff, 0x00, 0x01, 0x01, 0x00
fixup_table_end:
