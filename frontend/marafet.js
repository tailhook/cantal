var path = require('path')
var child = require('child_process')
module.exports = function(content) {
    this.cacheable()
    var callback = this.async()
    var pathname = path.relative(process.cwd(), this.resourcePath);
    var amd_name = path.dirname(pathname) + path.basename(pathname, '.mft')
    var ch = child.exec(
        "/usr/bin/marafet -f- --amd --auto-load-css --js -"
        + " --amd-name '" + amd_name + "'"
        + " --block-name 'b-" + path.basename(this.resourcePath, '.mft') + "'"
        , function(error, stdout, stderr) {
            if(stderr.length) {
                console.error(stderr)
            }
            if(error) {
                console.error(stdout)
                console.error(error)
                callback(error)
            } else {
                callback(null, stdout)
            }
        })
    ch.stdin.write(content)
    ch.stdin.end()
}
