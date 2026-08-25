import sys
import struct
from pathlib import Path


def extract_text_section(input_file, output_file):
    data = Path(input_file).read_bytes()

    # DOS header
    if data[:2] != b"MZ":
        raise ValueError("Not a PE file: missing MZ signature")

    pe_offset = struct.unpack_from("<I", data, 0x3C)[0]

    # PE signature
    if data[pe_offset:pe_offset + 4] != b"PE\0\0":
        raise ValueError("Invalid PE signature")

    file_header_offset = pe_offset + 4
    machine, number_of_sections, _, _, _, size_of_optional_header, _ = \
        struct.unpack_from("<HHIIIHH", data, file_header_offset)

    if machine != 0x8664:
        raise ValueError(f"Not a Windows x64 PE (machine=0x{machine:04X})")

    optional_header_offset = file_header_offset + 20
    section_table_offset = optional_header_offset + size_of_optional_header

    # IMAGE_SECTION_HEADER is 40 bytes
    for i in range(number_of_sections):
        offset = section_table_offset + i * 40

        name = data[offset:offset + 8].rstrip(b"\0").decode(
            "ascii", errors="replace"
        )

        virtual_size, virtual_address, raw_size, raw_offset = \
            struct.unpack_from("<IIII", data, offset + 8)

        if name == ".text":
            if raw_offset + raw_size > len(data):
                raise ValueError("Invalid .text section bounds")

            text_data = data[raw_offset:raw_offset + raw_size]
            Path(output_file).write_bytes(text_data)

            print(f"Extracted .text section")
            print(f"  Virtual address: 0x{virtual_address:X}")
            print(f"  Virtual size:    0x{virtual_size:X}")
            print(f"  Raw offset:      0x{raw_offset:X}")
            print(f"  Raw size:        0x{raw_size:X}")
            print(f"  Output:          {output_file}")
            return

    raise ValueError("No .text section found")


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input_file> <output_file>")
        sys.exit(1)

    try:
        extract_text_section(sys.argv[1], sys.argv[2])
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
