const zigbuild = @import("std").build;

fn createExeStep(b: *zigbuild.Builder, name: []const u8, rootFilePath: []const u8) *zigbuild.LibExeObjStep {
    const build_mode = b.standardReleaseOptions();
    const exe = b.addExecutable(name, rootFilePath);
    exe.setBuildMode(build_mode);
    exe.setOutputDir("bin");
    return exe;
}

pub fn build(b: *zigbuild.Builder) void {
    const all_step = b.step("all", "Build all executables");

    b.default_step.dependOn(all_step);
}
