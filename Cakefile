fs = require 'fs'
path = require 'path'
{promisify} = require 'util'

glob = promisify require 'glob'
rimraf = promisify require 'rimraf'
CoffeeScript = require 'coffeescript'

WasmPackPlugin = require '@wasm-tool/wasm-pack-plugin'
HtmlWebpackPlugin = require 'html-webpack-plugin'
webpack = require 'webpack'
WebpackDevServer = require 'webpack-dev-server'

dist = path.resolve __dirname, 'dist'

task 'clean:dist', 'clean out webpacked code', ->
  await rimraf 'dist'

task 'clean:pkg', 'clean out wasm compiled code', ->
  await rimraf 'pkg'

task 'clean:target', 'clean out cargo build output', ->
  await rimraf 'target'

compileCoffee = (coffeePath) ->
  input = await promisify(fs.readFile) coffeePath, encoding: 'utf8'
  jsPath = coffeePath.replace /\.coffee$/, '.js'
  console.log "compiling #{coffeePath} to #{jsPath}..."
  output = CoffeeScript.compile input,
    inlineMap: yes
    bare: yes
    header: yes
  await promisify(fs.writeFile) jsPath, output
  jsPath

task 'build:coffee', 'compile .coffee files', ->
  for appSource in await glob 'www/**/*.coffee'
    await compileCoffee appSource

option '-r', '--release', 'whether to produce a release bundle'

webpackConfig = ({release = no}) ->
  mode: if release then 'production' else 'development'
  entry:
    index: './www/index.js'
  output:
    path: dist
    filename: '[name].js'
  devServer:
    contentBase: dist
    port: 8080
  plugins: [
    new HtmlWebpackPlugin
      template: './www/index.html'
    new WasmPackPlugin
      crateDirectory: __dirname
      extraArgs: '--out-name index'
  ]

formatStats = (stats) ->
  stringified = stats.toString
    chunks: no                  # makes build quieter
    colors: yes
  if stats.hasErrors()
    throw new Error stringified
  stringified

task 'dist:webpack', 'run webpack compilation', ({release = no}) ->
  await invoke 'build:coffee'
  stats = await promisify(webpack) webpackConfig {release}
  console.log formatStats stats

webpackRunServer = promisify (cb) ->
  config = webpackConfig {release: no}
  compiler = webpack config
  devServerOptions = {config.devServer..., open: yes}
  server = new WebpackDevServer compiler, devServerOptions
  server.listen devServerOptions.port, devServerOptions.host, cb

task 'run:dev-server', 'run webpack dev server', ->
  await invoke 'build:coffee'
  await do webpackRunServer

task 'clean-build', 'clean and then rebuild the whole project', (options) ->
  await invoke 'clean:dist'
  await invoke 'clean:pkg'
  # await invoke 'clean:target'
  await invoke 'dist:webpack', options
