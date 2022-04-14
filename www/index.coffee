{promisify} = require 'util'

import('../pkg')
  .catch console.error
  .then ({beep, unbeep, rebeep, examine_bytes}) ->
    handle = null
    playButton = document.getElementById 'play'
    stopButton = document.getElementById 'stop'
    uploadButton = document.getElementById 'upload'

    playButton.addEventListener 'click', ->
      if handle?
        rebeep handle
      else
        handle = beep()

    stopButton.addEventListener 'click', ->
      if handle?
        unbeep handle
        # handle.free()
        # handle = null

    file = await new Promise (resolve, reject) ->
      handleFile = -> resolve @files[0]
      uploadButton.addEventListener 'change', handleFile, no

    console.log file
    buffer = new Uint8Array (await file.arrayBuffer())
    examine_bytes buffer
