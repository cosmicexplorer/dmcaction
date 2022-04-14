{promisify} = require 'util'

import('../pkg')
  .catch console.error
  .then ({beep, unbeep, rebeep, examine_file, play_recorded}) ->
    handle = null
    handleRec = null
    playButton = document.getElementById 'play'
    stopButton = document.getElementById 'stop'
    uploadButton = document.getElementById 'upload'
    playRecButton = document.getElementById 'play-recorded'

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

    playRecButton.addEventListener 'click', ->
      play_recorded handleRec if handleRec?

    console.log file
    buffer = new Uint8Array (await file.arrayBuffer())
    handleRec = examine_file file.name, file.type, buffer
