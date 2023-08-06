/**
 * @type {import('@rspack/cli').Configuration}
 */

module.exports = {
  context: __dirname,
  entry: {
    main: "./src/index.jsx",
  },
  output: {
    filename: "[name].[contenthash].bundle.js",
    publicPath: "/",
  },
  target: "web",
  builtins: {
    html: [
      {
        template: "./index.html",
        title: "Runtime.land | a tiny Function as a Service (FaaS) platform ",
        favicon: "./public/logo-v2-small.ico",
      },
    ],
    copy: {
      patterns: [
        /*{
          from: "src/config.js",
          to: "config.js",
        },*/
        {
          from: "public/**/*",
        },
      ],
    },
  },
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: "asset",
      },
      {
        test: /\.sass|.scss$/,
        use: [
          {
            loader: 'sass-loader',
            options: {
              // ...
            },
          },
        ],
        type: 'css',
      },
      {
        test: /\.png$/,
        type: "asset",
      },
      {
        test: /\.config.js/,
        type: "asset",
      },
    ],
  },
  devServer: {
    hot: true,
    historyApiFallback: true,
  },
};
