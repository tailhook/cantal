var webpack = require('webpack')
var DEV = process.env['NODE_ENV'] != 'production';
module.exports = {
    context: __dirname,
    entry: DEV ? [
        "./index",
        "webpack-dev-server/client?http://localhost:8080",
        "webpack/hot/only-dev-server",
    ] : "./index",
    output: {
        path: __dirname + "/../public/js",
        filename: "bundle.js",
        publicPath: '/js/',
    },
    module: {
        loaders: [{
            test: /\.khufu$/,
            loaders: ['babel', 'khufu'],
            exclude: /node_modules/,
        }, {
            test: /\.js$/,
            loaders: ['babel'],
            exclude: /node_modules/,
        }],
    },
    resolve: {
        root: ["/usr/local/lib/node_modules"],
    },
    resolveLoader: {
        root: ["/usr/local/lib/node_modules"],
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
    khufu: {
        static_attrs: !DEV,
    },
    plugins: [
        new webpack.NoErrorsPlugin(),
    ],
}

