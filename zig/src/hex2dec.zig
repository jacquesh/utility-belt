const std = @import("std");
const fmt = std.fmt;
const mem = std.mem;
const warn = std.debug.warn;

fn hex2int(hexChars: []u8) !u128 {
    var hexSlice = hexChars[0..];
    if (mem.eql(u8, hexSlice[0..2], "0x")) {
        hexSlice = hexChars[2..];
    }

    // NOTE: If not for the fact that we need to ignore dashes ('-') in the
    //       input, we could replace this logic with a call to fmt.parseInt()
    var val: u128 = 0;
    for (hexSlice) |c| {
        if (c == '-') {
            continue;
        }
        var charVal = try fmt.charToDigit(c, 16);
        val = (val * 16) + charVal;
    }

    return val;
}

pub fn main() !void {
    var arenaAlloc = std.heap.ArenaAllocator.init(std.heap.direct_allocator);
    defer arenaAlloc.deinit();
    const alloc = &arenaAlloc.allocator;

    const stdoutFile = try std.io.getStdOut();
    var stdout = stdoutFile.outStream().stream;

    var args = std.process.args();
    const program = args.next(alloc);

    var argsPresent = false;
    while (args.next(alloc)) |argErrorUnion| {
        if (argErrorUnion) |hexArg| {
            if (hex2int(hexArg)) |hexValue| {
                //warn("{}\n", hexValue);
                // TODO: This fails on Windows with "error: Unexpected":
                try stdout.print("{}\n", hexValue);
            } else |parseErr| {
                warn("Failed to parse hex value {}: {}\n", hexArg, parseErr);
            }
            argsPresent = true;
        } else |err| {
            warn("Error when attempting to read argument: {}\n", err);
            break;
        }
    }

    if (!argsPresent) {
        warn("Usage: {} INPUT...\n", program);
    }
}
