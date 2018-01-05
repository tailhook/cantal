var webpack = require('webpack')
var MinifyPlugin = require('babel-minify-webpack-plugin')
var DEV = process.env['NODE_ENV'] != 'production';
module.exports = {
    context: __dirname,
    entry: DEV ? [
        "./index",
        //"webpack-dev-server/client?http://localhost:8080",
        //"webpack/hot/only-dev-server",
    ] : "./index",
    output: {
        path: __dirname + "/../public/js",
        filename: "bundle.js",
        publicPath: '/js/',
    },
    module: {
        loaders: [{
            test: /\.khufu$/,
            loaders: ['babel-loader', 'khufu'],
            exclude: /node_modules/,
        }, {
            test: /\.js$/,
            loaders: ['babel-loader'],
            exclude: /node_modules/,
        }],
    },
    resolve: {
        modules: process.env.NODE_PATH.split(':').filter(x => x),
    },
    resolveLoader: {
        mainFields: ["webpackLoader", "main", "browser"],
        modules: process.env.NODE_PATH.split(':').filter(x => x),
    },
    devServer: {
        contentBase: '../public',
        //contentBase: 'http://localhost:8080/',
        proxy: {
            '/*.json': {
                target: 'http://localhost:22682',
                secure: false,
            },
            '/*.cbor': {
                target: 'http://localhost:22682',
                secure: false,
            },
        },
        publicPath: '/js/',
        hot: true,
        historyApiFallback: true,
    },
    plugins: [
        new webpack.LoaderOptionsPlugin({
            options: {
                khufu: {
                    static_attrs: !DEV,
                },
                babel: {
                    "plugins": [
                        "transform-strict-mode",
                        "transform-object-rest-spread",
                        "transform-es2015-block-scoping",
                        "transform-es2015-parameters",
                        "transform-es2015-destructuring",
                        "transform-es2015-arrow-functions",
                        "transform-es2015-block-scoped-functions",
                    ],
                },
            }
        }),
        new webpack.NoEmitOnErrorsPlugin(),
        new webpack.DefinePlugin({
            VERSION: JSON.stringify("0.5.11"),
            "process.env.NODE_ENV": JSON.stringify(process.env['NODE_ENV']),
            DEBUG: DEV,
        }),
    ].concat(DEV ? [] : [
        new MinifyPlugin({}),
    ]),
}

