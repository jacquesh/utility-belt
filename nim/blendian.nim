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


proc printValue[T: SomeInteger](input: string, printWordHeader: bool, allowLE: bool, allowBE: bool) =
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

    assert allowLE or allowBE
    let printEndianHeader = allowLE and allowBE

    if printWordHeader:
        stdout.writeLine(fmt"{sizeof(T)*8}-bit words:")

    if allowBE:
        if printEndianHeader:
            stdout.write("   Big endian: ")
        printSequence(vals_be)

    if allowLE:
        if printEndianHeader:
            stdout.write("Little endian: ")
        printSequence(vals_le)

    stdout.write("\n")


proc main() =
    let args = os.commandLineParams()
    if (len(args) == 0) or (args[0] == "--help"):
        stderr.writeLine("BLendian: Display hex strings as a sequence of integers in both big- and little endian")
        stderr.writeLine("")
        stderr.writeLine("Usage: blendian [-16|-32|-64] [-be|-le] <hex-string>")
        quit(1)

    let only16 = ("-16" in args)
    let only32 = ("-32" in args)
    let only64 = ("-64" in args)
    let onlyLE = ("-le" in args)
    let onlyBE = ("-be" in args)

    if (only16 and only32) or (only16 and only64) or (only32 and only64):
        stderr.writeLine("Cannot specify 2 different word sizes to display, only one of -16, -32, or -64 is allowed")
        quit(1)

    let sizeSpecified = (only16 or only32 or only64)
    let allow16 = only16 or not sizeSpecified
    let allow32 = only32 or not sizeSpecified
    let allow64 = only64 or not sizeSpecified

    let allowLE = onlyLE or (not onlyLE and not onlyBE)
    let allowBE = onlyBE or (not onlyLE and not onlyBE)
    assert allowLE or allowBE

    # TODO: Also, take this from stdin so that we can pipe things together?
    for input in args:
        if input.startsWith('-'):
            continue

        var hex = input
        hex.removePrefix("0x")
        if len(hex) mod 2 == 1:
            hex = "0" & hex

        if allow16:
            printValue[uint64](hex, not sizeSpecified, allowLE, allowBE)

        if allow32:
            printValue[uint32](hex, not sizeSpecified, allowLE, allowBE)

        if allow64:
            printValue[uint16](hex, not sizeSpecified, allowLE, allowBE)

main()
