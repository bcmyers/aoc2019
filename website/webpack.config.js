const path = require('path')

const CopyPlugin = require('copy-webpack-plugin')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')

const dist = path.resolve(__dirname, 'dist')

module.exports = {
  mode: 'production',
  entry: {
    index: './js/index.js',
  },
  output: {
    path: dist,
    filename: '[name].js',
  },
  devServer: {
    contentBase: dist,
  },
  module: {
    rules: [
      {
        test: /\.(js|jsx|ts|tsx)$/,
        loader: require.resolve('babel-loader'),
        options: {
          customize: require.resolve(
            'babel-preset-react-app/webpack-overrides',
          ),
        },
      },
    ],
  },

  plugins: [
    new CopyPlugin([path.resolve(__dirname, 'js', 'index.html')]),
    new WasmPackPlugin({
      crateDirectory: __dirname,
      extraArgs: '--out-name index',
    }),
  ],
}
