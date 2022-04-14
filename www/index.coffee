import('../pkg')
  .catch console.error
  .then (rustModule) ->
    handle = null
    playButton = document.getElementById 'play'
    stopButton = document.getElementById 'stop'

    playButton.addEventListener 'click', ->
      if handle?
        rustModule.rebeep handle
      else
        handle = rustModule.beep()

    stopButton.addEventListener 'click', ->
      if handle?
        rustModule.unbeep handle
        # handle.free()
        # handle = null
