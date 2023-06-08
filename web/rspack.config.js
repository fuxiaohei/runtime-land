/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  context: __dirname,
  entry: {
    main: "./src/main.jsx",
  },
  builtins: {
    html: [
      {
        template: "./index.html",
        title: "Moni Serverless",
        favicon: "./public/favicon.png",
      },
    ],
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
    ],
  },
  devServer: {
    hot: true,
    historyApiFallback: true,
  },
};
