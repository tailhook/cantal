var fs = require('fs')
var jade = require('jade')
var babel = require('babel')
var uglify = require("uglify-js")

const PRELUDE = "import {handle, merge_events} from 'util/events'"

class Compiler {
    constructor(node, options) {
        this.node = node
        this.options = options
    }
    compile() {
        console.assert(this.node.type == "Block", "Block", this.node.type)
        var code = this.node.nodes.filter(x => x.type == 'Code')
        var tags = this.node.nodes.filter(x => x.type == 'Tag')
        var mixins = this.node.nodes.filter(x => x.type == 'Mixin')
        if(tags.length == 0) {
            return this.compile_mixins(code, mixins)
        } else {
            return this.compile_template(code, tags, mixins)
        }
    }
    compile_mixins(code, mixins) {
        var mixins = mixins.map(m => {
            return `export function ${ m.name }(${ m.args }) {
                return (__block_function__) => {
                    return ${ this.visit_fragment(m.block) }
                }}`
        })
        var chunks = code.map(x => x.val)
        chunks.splice(0, 0, PRELUDE)
        chunks = chunks.concat(mixins)
        return chunks.join('\n')
    }
    compile_template(code, tags, mixins) {
        var mixins = mixins.map(m => {
            return `var ${ m.name } = (${ m.args }) => {
                return (__block_function__) => {
                    return ${ this.visit_fragment(m.block) }
                }}`
        })
        if(tags.length > 1) {
            var body = `[${ tags.map(x => this.visit(x)).join(',') }]`
        } else {
            var body = this.visit(tags[0])
        }
        var chunks = code.map(x => x.val)
        chunks.splice(0, 0, PRELUDE)
        chunks.push(`exports.render = function render() {`+
            mixins.join('\n') +
            `\n return ${body}; }`)
        return chunks.join('\n')
    }
    visit(node) {
        var meth = this['visit_' + node.type.toString().toLowerCase()];
        if(!meth) {
            console.error("Can't process node type", node.type, node)
            throw Error("Can't process block")
        }
        return meth.call(this, node)
    }
    render_block(node) {
        var res = []
        var nodes = node.nodes.concat()
        while(nodes.length) {
            var item = nodes.shift()
            if(item.type == 'Code') {
                if(item.val.substr(0, 3) == 'if ') {
                    var body = this.visit_fragment(item.block)
                    var cond = `(${ item.val.substr(2) } ? ${ body }`
                    var else_pending = true;
                    while(nodes.length && nodes[0].type == 'Code' &&
                          nodes[0].val.substr(0, 4) == 'else') {
                        var item = nodes.shift()
                        cond += ': ';
                        var body = this.visit_fragment(item.block)
                        if(item.val.substr(5, 3) == 'if ') {
                            cond += `(${ item.val.substr(8) }) ? ${ body }`
                        } else {
                            cond += `${ body }`
                            else_pending = false;
                            break
                        }
                    }
                    if(else_pending) {
                        cond += ': ""'
                    }
                    cond += ")"
                    res.push(cond)
                } else {
                    res.push(this.item)
                }
            } else {
                var item = this.visit(item)
                if(item) {
                    res.push(item)
                }
            }
        }
        return res
    }
    visit_block(node) {
        var res = this.render_block(node)
        return `[${ res.join(', ') }]`
    }
    visit_mixinblock(node) {
        return '__block_function__()'
    }
    visit_fragment(node) {
        var items = this.render_block(node)
        if(items.length == 1) {
            return items[0]
        } else {
            return `{children:[${ items.join(',') }]}`
        }
    }
    visit_item(block) {
        if(block.nodes.length == 1) {
            return this.visit(block.nodes[0])
        } else {
            var items = block.nodes.map(x => this.visit(x)).join(',')
            return `{children: [${items}]}`
        }
    }
    visit_mixin(node) {
        if(node.block) {
            var block = this.visit_fragment(node.block)
            return `${ node.name }(${ node.args })((context) => {
                return ${ block }
            })`
        } else {
            return `${ node.name }(${ node.args })()`
        }
    }
    visit_attributes(attrs, blocks) {
        var pairs = attrs
            .filter(x => x.name != 'class' && x.name[0] != '+')
            .map(x => `${x.name}: ${x.val}`)
        var events = attrs
            .filter(x => x.name[0] == '+' && x.name[1] != '+')
            .map(x => {
                const name = x.name.substr(1)
                if(x.val === true) {
                    return `${name}: handle(this.on_${name}.bind(this))`
                } else {
                    return `${name}: ${x.val}`
                }
            })
        var extra_events = attrs
            .filter(x => x.name.substr(0, 2) == '++')
            .map(x => {
                const name = x.name.substr(2)
                if(x.val === true) {
                    return `${name}(this)`
                } else {
                    throw Error("Don't use equals after spread operator")
                }
            })
        if(extra_events.length == 0) {
            var ev_str = `{${events.join(', ')}}`
        } else {
            var ev_str = `merge_events({${ events.join(', ') }},
                ${ extra_events.join(', ') })`
        }
        var classes = attrs
            .filter(x => x.name == 'class')
            .map(x => (!x.escaped)
                ? x.val.replace(/^'|'$/g, '')
                : "'+(" + x.val + ")+'")
        if(classes.length) {
            pairs.unshift(`class: '${classes.join(' ')}'`)
        }
        return [`{${pairs.join(', ')}}`, ev_str]
    }
    visit_code(node) {
        return node.val
    }
    visit_comment(node) {
    }
    visit_tag(node) {
        if(node.code) {
            console.assert(!node.block.length, "Tag", node)
            var children = `String(${ node.code.val })`
        } else {
            var children = this.visit_block(node.block)
        }
        var [attrs, events] = this.visit_attributes(
            node.attrs, node.attributeBlocks)
        // TODO(tailhook) escape tag name
        return `{
            tag: "${node.name}",
            attrs: ${attrs},
            events: ${events},
            children: ${children},
            }`
    }
    visit_each(node) {
        return (`{children: (${ node.obj }).map(` +
                `(${ node.val }, ${ node.key }) ` +
                `=> { return ${ this.visit_fragment(node.block) }; })}`)
    }
    visit_text(node) {
        // TODO(tailhook) escape text
        return `"${node.val}"`
    }
}


var bundle = []
for(var i = 2, il = process.argv.length; i < il; ++i) {
    var filename = process.argv[i]
    var module_name = filename.substr(0, filename.length - 5)  // strip .jade
    var src = fs.readFileSync(filename).toString()

    try {
        var node = new jade.Parser(src, filename, {}).parse()
    } catch (e) {
        console.error("Jade syntax error", e)
        process.exit(1)
    }
    try {
        var js = new Compiler(node).compile()
    } catch (e) {
        console.error("Jade compilation error", e.stack)
        process.exit(1)
    }
    try {
        var pretty = babel.transform(js, {
            stage: 0,
            filename: filename,
            modules: "amd",
            moduleIds: true,
            }).code
    } catch (e) {
        console.error("Error beautifying code", e)
        console.log("CODE", js)
        process.exit(1)
    }

    bundle.push(pretty)
}

fs.writeFileSync('../public/js/templates.js', bundle.join('\n'))
