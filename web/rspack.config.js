/**
 * @type {import('@rspack/cli').Configuration}
 */

module.exports = {
  context: __dirname,
  entry: {
    main: "./src/main.jsx",
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
        favicon: "./public/runtime-land-logo-32.ico",
      },
    ],
    copy: {
      patterns: [
        {
          from: "src/config.js",
          to: "config.js",
        },
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
