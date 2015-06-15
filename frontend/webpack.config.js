module.exports = {
    entry: {
        app: "./app/main.js",
    },
    output: {
        path: "../public/js",
        filename: "app.js",
    },
    resolveLoader: {
        modulesDirectories: [
            "/usr/local/lib/node_modules",
            ".",
        ],
    },
    resolve: {
        modulesDirectories: [
            "/work/frontend",
        ],
    },
    module: {
      loaders: [
         {
           test: /\.js$/,
           exclude: /(node_modules|bower_components)/,
           loader: 'babel-loader',
           query: {
             //optional: ['runtime'],
             stage: 0
           }
         },
         {
           test: /\.mft$/,
           loader: 'marafet',
         },
      ],
    },
}
