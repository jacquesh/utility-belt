const std = @import("std");
const mem = std.mem;
const warn = std.debug.warn;

const StringConvertError = error{InvalidHexCharacter};

fn charToDigit(char: u8) !u8 {
    return switch (char) {
        '0'...'9' => char - '0',
        'a'...'f' => char + 10 - 'a',
        'A'...'F' => char + 10 - 'A',
        else => StringConvertError.InvalidHexCharacter,
    };
}

// TODO: This can actually be replaced with fmt.parseInt(u128, hexChars, 16) but then we need a string.replace function to remove the dashes
fn hex2int(hexChars: []u8) !u128 {
    var hexSlice = hexChars[0..];
    if (mem.eql(u8, hexSlice[0..2], "0x")) {
        hexSlice = hexChars[2..];
    }

    var val: u128 = 0;
    for (hexSlice) |c| {
        if (c == '-') {
            continue;
        }
        var charVal = try charToDigit(c);
        val = (val * 16) + charVal;
    }

    return val;
}

pub fn main() !void {
    var kernelAlloc = std.heap.DirectAllocator.init();
    var arenaAlloc = std.heap.ArenaAllocator.init(&kernelAlloc.allocator);
    defer kernelAlloc.deinit();
    defer arenaAlloc.deinit();
    const alloc = &arenaAlloc.allocator;

    const stdoutFile = try std.io.getStdOut();
    var stdout = stdoutFile.outStream().stream;

    var args = std.os.args();
    const program = args.next(alloc);
    // TODO: I'm fairly certain this would skip the first argument on linux? Seemed to work without it...
    //warn("Program name is {}\n", program);

    var argsPresent = false;
    while (true) {
        // TODO: This can be merged into the while loop header (search docs for 'while with optionals' and 'while with error unions')
        const hexArg = try args.next(alloc) orelse break;
        const argValue = hex2int(hexArg);
        warn("{}\n", argValue);
        // TODO: This fails on Windows with "error: Unexpected": try stdout.print("{}\n", argValue);
        argsPresent = true;
    }

    if (!argsPresent) {
        warn("Usage: hex2dec INPUT...\n");
    }
}
