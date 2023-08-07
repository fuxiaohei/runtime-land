/**
 * @type {import('@rspack/cli').Configuration}
 */

let api_url = (process.env.NODE_ENV === 'development') ? "http://api-dev.127-0-0-1.nip.io" : "https://center.runtime.land";
if (process.env.API_URL) {
  api_url = process.env.API_URL;
}

console.log("API_URL:", api_url);

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
        {
          from: "public/**/*",
        },
      ],
    },
    define: {
      API_URL: "'" + api_url + "'",
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
