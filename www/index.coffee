import('../pkg')
  .catch console.error
  .then (rustModule) ->
    handle = null
    playButton = document.getElementById 'play'
    stopButton = document.getElementById 'stop'

    playButton.addEventListener 'click', ->
      handle = rustModule.beep()

    stopButton.addEventListener 'click', ->
      if handle?
        handle.free()
        handle = null
