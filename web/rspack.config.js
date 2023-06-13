/**
 * @type {import('@rspack/cli').Configuration}
 */

const isProduction = process.env.NODE_ENV === "production";

module.exports = {
  // mode: isProduction ? "production" : "development",
  context: __dirname,
  entry: {
    main: "./src/main.jsx",
  },
  output: {
    filename: "[name].[contenthash].bundle.js",
  },
  target: "web",
  builtins: {
    html: [
      {
        template: "./index.html",
        title: "Runtime.land | a tiny Function as a Service (FaaS) platform ",
        favicon: "./public/favicon.png",
      },
    ],
    define: {
      GRPC_ADDR: isProduction
        ? "'https://grpcapi.runtime.land'"
        : "'http://127.0.0.1:38779'",
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
    ],
  },
  devServer: {
    hot: true,
    historyApiFallback: true,
  },
};
