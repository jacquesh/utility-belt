import os
import parseutils
import strutils
import std/endians
import std/strformat
import terminal

proc parseBigEndianHex[T: SomeInteger](input: string, start=0, maxLen=0): tuple[big: T, little: T] =
    var val_ne: T
    let chars_parsed = parseutils.parseHex(input, val_ne, start, maxLen)
    assert chars_parsed > 0
    var val_oe: T # "Other endian"
    when sizeof(T) == 8:
        endians.swapEndian64(addr val_oe, addr val_ne)
    elif sizeof(T) == 4:
        endians.swapEndian32(addr val_oe, addr val_ne)
    elif sizeof(T) == 2:
        endians.swapEndian16(addr val_oe, addr val_ne)
    return (big: val_ne, little: val_oe)

proc printSequence[T: SomeInteger](vals: seq[T]) =
    var second = false
    for val in vals:
        if isatty(stdout):
            if second:
                stdout.setForeGroundColor(fgCyan)
            else:
                stdout.setForeGroundColor(fgGreen)
            second = not second

        when sizeof(T) == 8:
            stdout.write(fmt"{val:016x} ")
        elif sizeof(T) == 4:
            stdout.write(fmt"{val:08x} ")
        elif sizeof(T) == 2:
            stdout.write(fmt"{val:04x} ")


    stdout.resetAttributes()
    stdout.write("\n")


proc printValue[T: SomeInteger](input: string) =
    assert len(input) mod 2 == 0
    if len(input) mod sizeof(T)*2 != 0:
        stderr.write("TODO: No 2-byte output!")
        return

    var vals_be = newSeq[T](0)
    var vals_le = newSeq[T](0)
    var start_index = 0
    while start_index < len(input):
        let (val_be, val_le) = parseBigEndianHex[T](input, start=start_index, maxLen=sizeof(T)*2)
        start_index += sizeof(T)*2
        vals_be.add(val_be)
        vals_le.add(val_le)

    stdout.writeLine(fmt"{sizeof(T)*8}-bit words:")
    stdout.write("   Big endian: ")
    printSequence(vals_be)

    stdout.write("Little endian: ")
    printSequence(vals_le)

    stdout.write("\n")


proc main() =
    let args = os.commandLineParams()
    if (len(args) == 0) or (args[0] == "--help"):
        stderr.writeLine("blendian: Display hex strings as a sequence of integers in both big- and little endian")
        stderr.writeLine("")
        stderr.writeLine("Usage: blendian <hex-string>")
        return

    # TODO: Also, take this from stdin so that we can pipe things together?
    # TODO: Allow the user to specify an endianness, base and int size, and then only output those things, so this can be piped to other programs?
    var input = args[0]
    input.removePrefix("0x")
    if len(input) mod 2 == 1:
        input = "0" & input

    printValue[uint64](input)
    printValue[uint32](input)
    printValue[uint16](input)

main()
