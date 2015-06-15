var path = require('path')
var child = require('child_process')
module.exports = function(content) {
    this.cacheable()
    var callback = this.async()
    var ch = child.exec(
        "../marafet -f- --amd --js -"
        + " --amd-name '" + this.resourcePath + "'"
        + " --block-name 'b-" + path.basename(this.resourcePath, '.mft') + "'"
        , function(error, stdout, stderr) {
        if(error) {
            console.error(error)
        }
        if(stderr.length) {
            console.error(stderr)
        }
        callback(null, stdout)
    })
    ch.stdin.write(content)
    ch.stdin.end()
}
