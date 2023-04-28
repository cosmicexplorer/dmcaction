{promisify} = require 'util'
{readFile} = require 'fs/promises'

pkg = import("../pkg.wasm")
console.log pkg

# wasmBytes = await readFile("./pkg.wasm")
# resp = new Response wasmBytes,
#   headers:
#     'Content-Type': 'application/wasm'
# console.log resp
# wasmLib = await WebAssembly.instantiateStreaming resp, {imports: {}}
# wasmLib = await WebAssembly.instantiate wasmBytes, {imports: {}}

# wasmLib.instance.exports.log_result()

# import('../pkg.wasm')
#   .catch console.error
#   .then ({beep, unbeep, rebeep, examine_file, play_recorded, log_result}) ->
#     log_result()

#     handle = null
#     playButton = document.getElementById 'play'
#     stopButton = document.getElementById 'stop'
#     uploadButton = document.getElementById 'upload'
#     link = document.getElementById 'link'

#     playButton.addEventListener 'click', ->
#       if handle?
#         rebeep handle
#       else
#         handle = beep()

#     stopButton.addEventListener 'click', ->
#       if handle?
#         unbeep handle
#         # handle.free()
#         # handle = null

#     file = await new Promise (resolve, reject) ->
#       handleFile = -> resolve @files[0]
#       uploadButton.addEventListener 'change', handleFile, no

#     console.log file
#     buffer = new Uint8Array (await file.arrayBuffer())
#     extractedData = examine_file file.name, file.type, buffer
#     b64Encoded = Buffer.from(extractedData).toString 'base64'
#     link.setAttribute 'href', "data:audio/m4a;base64,#{b64Encoded}"
#     link.setAttribute 'download', file.name
#     link.click()
