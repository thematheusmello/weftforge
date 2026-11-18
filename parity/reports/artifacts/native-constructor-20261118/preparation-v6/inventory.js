ObjC.import('AppKit');
ObjC.import('CoreGraphics');
function run(argv) {
    var all = $.NSWorkspace.sharedWorkspace.runningApplications;
    var apps = [], childApps = [], childPid = argv.length > 1 ? Number(argv[1]) : null;
    for (var i = 0; i < all.count; i++) {
        var app = all.objectAtIndex(i);
        var pid = Number(app.processIdentifier), bundle = ObjC.unwrap(app.bundleIdentifier);
        if (bundle === argv[0] || pid === childPid) {
            var launchTime = Number(app.launchDate.timeIntervalSince1970);
            var identity = {pid: pid, executable: ObjC.unwrap(app.executableURL.path),
                       bundle_id: ObjC.unwrap(app.bundleIdentifier), app_path: ObjC.unwrap(app.bundleURL.path),
                       launch_time: Number.isFinite(launchTime) ? launchTime : null};
            if (bundle === argv[0]) apps.push(identity);
            if (pid === childPid) childApps.push(identity);
        }
    }
    var ids = apps.map(function(app) { return app.pid; });
    if (childPid !== null && ids.indexOf(childPid) < 0) ids.push(childPid);
    var windows = ObjC.deepUnwrap(ObjC.castRefToObject($.CGWindowListCopyWindowInfo(0, 0)));
    return JSON.stringify({apps: apps, child_apps: childApps, windows: windows.filter(function(window) {
        return ids.indexOf(window.kCGWindowOwnerPID) >= 0;
    })});
}
