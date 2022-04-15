DMCAction
=========

**This codebase intends to produce a tool to strip an audio track from an IRL recording.**

# Goals

The use case is specifically:
1. The recording is from a mobile device, with an imperfect microphone.
2. The audio track to be stripped from the recording is being played IRL, from an imperfect speaker.
3. The audio track corresponds to a well-known piece for which there are only a few "canonical" recordings.
    - This means that you can expect the audio track being played IRL to come directly from a recorded waveform that is possible to locate online.

## Significance

**The goal of this tool is to make it impossible for someone to play a copyrighted track while performing something they don't want on video.** This technique has been used to trigger overzealous automated enforcement of the Digital Millennium Copyright Act (DMCA)[^dmca] on video hosting platforms such as youtube, which causes the video to be immediately removed without any chance of appeal[^santa-ana-police-youtube]. This is one insidious way that certain people censor recordings of their foul acts and therefore escape justice. It is one small component of a technical infrastructure to protect protestors and anyone else facing violence with a mobile phone. *You are extremely important.*

Of course, since protestors are not the only ones facing unfair censorship via the DMCA, this tool may also have wider applications.

This project refers to the DMCA[^dmca], which is a US law that defines copyright in a way that is extremely friendly to litigious copyright holders and incompatible with most ways people interact with popular culture. As described above, it is also extremely useful for automating censorship at the state level. **This tool should not have to exist.**

## Overview

Given an *IRL recording* we wish to strip *copyrighted audio* from, we imagine a process to perform that stripping below (items marked with *(tentative)* are suspected to not be very useful):

1. [ ] [selection of canonical copyrighted audio](#selection-of-canonical-audio)
    1. [ ] *(tentative)* [shazam-style fingerprint identification](#audio-fingerprinting)
    2. [ ] [selecting among recordings to find the best canonical track](#canonical-track-selection)
2. [ ] [track alignment](#track-alignment)
    1. [ ] [phase shift (identify start of track)](#phase-shift)
    2. [ ] *(tentative)* [dilation (identify whether audio is stretched or compressed in time)](#time-dilation)
    3. [ ] [amplitude (identify where audio from canonical recording is louder or softer)](#amplitude-scaling)

It is expected that after *track alignment* is performed, that the modified *canonical recording* can be inverted[^wave-inversion], then added to the audio from the *IRL recording*. This synthesis is intended to produce an *output audio track* which does not contain any amount of the *copyrighted audio* which a human or automated censor could detect.

Ideally, the *output audio track* would not be found to constitute a copyright infringement by a jury in a court of law as well; however, the more important goal is to confound automated tools, since those can be applied at much greater scale and in far less accountable ways.

### TODO

**Currently, while we can successfully read in a `.m4a` audio file with `symphonia`, and even play it without loss of quality via `cpal`, bouncing the underlying PCM waveform to something like a `.wav` file with any of the rust wav implementations leads to extremely severe degradation of audio quality.** This is minimized when using `f32`s to store samples (vs e.g. `i16`s), but is still completely unusable.

To recap, the intent is a web application which processes audio uploaded using webassembly, all directly on the client. While most of the work revolves around analysis which can be done by staying within the easier-to-manipulate PCM waveform formats, when we actually produce an output file, we'll want it to be as high quality as the original input audio, and ideally in the exact same format. This is to minimize the amount of specialized software or expertise needed to use this application.

Since we are able to play the audio we extracted from `symphonia` with `cpal`, we can trust `symphonia` is not destroying the audio quality, but rather every single library we've found to convert to `.wav` files is trashing the quality somehow. This is a solvable problem; at worst, we can implement encoders for the formats `symphonia` so far only has *decoders* for. That may be the least effort and most reliable way out of this, as `symphonia` appears to be an extremely high quality library and would likely appreciate the effort.

# Selection of Canonical Audio

As described in [goals](#goals), we assume that the audio which is being played IRL is one for which there exists at most one or two *canonical recordings*. Most recorded music is expected to have this property, and this is part of why automated systems are able to recognize it in the first place.

## Audio Fingerprinting

*(tentative)*

While we have narrowly defined our [goals](#goals) to assume that the copyrighted audio being played IRL is easy to identify, this may not always be the case. If a copyrighted track is not easy to identify, we could consider a database of "fingerprints" similar to shazam's approach [^shazam-annoyingly-sparse-paper]. However, since shazam or an equivalent is widely available on most mobile devices, this feature is likely unnecessary compared to the immense difficulty of curating such a database.

## Canonical Track Selection

If there is more than one unique canonical recording of a work, the software should be able to:
1. perform the [alignment](#track-alignment) process on all canonical recordings,
2. identify the recording with the greatest *degree of alignment*.

## Aside: Handling Live Performances

Asserting copyright over recordings of live performances via the DMCA induces additional challenges:
- Classical musicians face DMCA takedowns over *their own performance* of sheet music which itself is public domain[^bach-youtube-dmca]. The performer of course owns the copyright to their own recording of a public domain work, so censoring it on copyright grounds (and worse, monetizing the video for the claimed copyright owner, not the performer) is actually a form of fully automated copyright infringement for which there exists no easy remedy without a lawsuit.
- The copyright from recordings of live performances in general will legitimately belong to the performers, and can be considered a violation of DMCA if an audio track is uploaded to an internet service containing any recognizable part of that live recording. However, the *canonical recording* for the work may be significantly more difficult or impossible to obtain. **This requires additional work to support, and might be the motivation to consider more principled [time-frequency analysis](#time-frequency-analysis).**

# Track Alignment

The track alignment process solves for multiple parameters which transform the canonical recording most closely to its appearance in the IRL recording. The *aligned waveform* of the canonical recording after these transformations can then be inverted and subtracted from the IRL recording to produce the output audio track. **The parameters listed in this section are heuristics[^heuristic], and may not yet represent all of the ways that a recording can be distorted when recorded IRL.**

We currently make a few simplifying assumptions (which can be removed later as necessary):
1. The IRL recording contains only one instance of copyrighted audio to be removed.
2. The IRL recording plays the copyrighted audio contiguously (without any gaps).
    - For example, if the IRL speaker producing the copyrighted audio stopped and started its playback, this assumption would be wrong.
3. The IRL recording plays the copyrighted audio at a constant speed/tempo.

The *degree of alignment* is a measure of how much *amplitude over time* decreases after the *aligned waveform* is subtracted from the IRL recording. We aim to identify these *alignment parameters* and calculate the *degree of alignment* via an [optimization process](#alignment-methods).

## Phase Shift

The *phase shift*[^phase] here is used as an umbrella term to refer to all of:
- the time within the IRL recording that the copyrighted audio begins.
- the time within the IRL recording that the copyrighted audio ends.
- the time within the canonical recording that the IRL recording begins at.
- the time within the canonical recording that the IRL recording ends at.

## Time Dilation

*(tentative)*

The *time dilation* is a measure of how much the canonical recording has been stretched or compressed in time. Since speeding up or slowing down the recording is a known workaround for youtube's overzealous DMCA censorship **(TODO: add footnote!)**, it is not currently expected that this transformation would be used on a canonical recording, so we ignore it for now.

## Amplitude Scaling

The *amplitude scaling* is not a single number, but a sequence of scaling factors `0 <= a_t <= \infty` to apply to consecutive frames of the canonical recording that minimizes the *amplitude over time* when subtracted from the IRL recording.

We could make a few assumptions about where this amplitude scaling arises from:
- increasing the volume on the IRL speaker producing copyright audio.
- moving the IRL speaker closer to the IRL microphone.

See [alignment methods](#alignment-methods) for further simplifying assumptions we make to this parameter, including the assumption of a *global constant amplitude scaling factor* (which can be relaxed).

# Alignment Methods

There are two competing methods currently under consideration to identify *alignment parameters*. **Any further comments or suggestions are welcome!**

## Time-Frequency Analysis

Time-frequency analysis[^time-freq-analysis] is the more "scientific" way to do this, and offers multiple methods:
1. Least-squares spectral analysis[^lssa].
2. Wavelets[^wavelet].

Of the two, least-squares spectral analysis seems less complex to implement from scratch, but seems to have less prior art on its use. On the other hand, the GNU scientific library implements wavelet transforms already, with fantastic documentation[^gsl-wavelets].

**However, we likely won't attempt to understand and use either of these techniques, because we have assume we have access to a canonical recording. This should enable a much simpler approach which we describe in [brute-force optimization](#brute-force-optimization).**

## Brute Force Optimization

Because we assume access to a canonical recording, we can employ some methods which may not work [if the automated detection system catches "fuzzy" matches of live performances](#aside-handling-live-performances). In particular, we can:
1. split the canonical recording and IRL recording into small time slices.
    - Preferably, this time slice is the exact sampling frequency of the underlying audio tracks. However, if the audio tracks do not share the same sampling frequency, one will need to be converted into the other. This should typically involve taking the higher-quality recording (expected to be the canonical recording) and downsampling it to the frequency of the lower-quality recording (expected to be the IRL recording). Once the two tracks share a sampling frequency, that frequency can be used as the width of the time slice.
2. calculate the *amplitude over time* by summing the instantaneous amplitude which is assigned to each time slice in the audio file.
3. choose arbitrary parameters for [phase shift](#phase-shift) and assume [amplitude scaling](#amplitude-scaling) is set to a scale factor of 1 at all times.
    - For example, we could choose to "center" the canonical recording within the IRL recording as an initial guess. However, we would probably accept user input to speed along this process; see [steps](#steps).
4. calculate the *degree of alignment* by inverting the current *aligned waveform*, adding the inverted aligned waveform to the IRL recording, then calculating the *amplitude over time* of the output audio track.
    - At first, this number is very likely to be negative, meaning that adding the inverted aligned waveform to the IRL recording produces a result that has *more sound* than the original (this would sound like two versions of the song playing in a canon[^canon-music] at once).
5. perform a simple optimization scheme to maximize the *degree of alignment* by perturbing the current *alignment parameters*.
    - **We expect that the *degree of alignment* as a fitness function of our *alignment parameters* is not a convex function[^convex-function]**, but instead has a single global maximum which has a much much higher value than any other point (but it would not be possible to find "neighboring" points that have slightly better values).

### Steps

Given the above, we develop a multi-stage optimization process:
1. Allow user to input estimates for all [phase shift](#phase-shift) parameters.
2. Given those initial estimates, perform an exhaustive search of the state space for phase shift parameters "neighboring" the user-provided ones, and identify the one with the maximum degree of alignment.
    - **At this point, our current output audio track should have a smaller amplitude over time than the original IRL recording.** If the amplitude over time is not decreased by some small threshold value, we will exit and report to the user that alignment was impossible.
3. Then identify the [amplitude scaling](#amplitude-scaling) by iterating over each frame shared by the currently transformed canonical recording and original IRL recording.
    - First attempt to identify a *global amplitude scaling factor* which maximizes the degree of alignment by performing an ad-hoc gradient descent[^gradient-descent] which multiplies the signal level of each frame of the currently transformed canonical recording before adding it to the IRL recording. The initial guess for this factor is just 1.
    - **We assume that the amplitude scaling over time does not change, and instead ask the user to split up the IRL recording into separate tracks, each of which is assumed to have a constant amplitude scaling factor. We will provide an interface that makes this easy.**
4. Finally check if the resulting output audio track has an amplitude over time which is less than the original by some small threshold value (but larger than the previous threshold); if not, exit and report to the user a failure to satisfactorily align the audio. Otherwise, provide the output audio track to the user.

# UX Considerations

Ideally this is a web application using javascript or WASM[^wasm] so that it can be accessed via any internet connection. This application should perform no network calls and perform all calculation strictly on the client. This application should be able to accept most common audio and video formats, and it should return a file of the exact same length as the original **(NOTE: this means we should only ever scale the sampling frequency of the canonical recording, not the IRL recording!)**, in the same audio or video format.

## Iteration

However, we can begin this process by creating a CLI application using command-line tools such as `ffmpeg`[^ffmpeg] to perform some of our conversions. We intend to eventually replace all outside applications with uses of pure-rust libraries so that the result can be compiled to WASM.

[^dmca]: https://en.wikipedia.org/wiki/Digital_Millennium_Copyright_Act
[^santa-ana-police-youtube]: https://twitter.com/hackermaderas/status/1512216043053367305
[^wave-inversion]: https://en.wikipedia.org/wiki/Phase_%28waves%29#For_sinusoids_with_same_frequency
[^bach-youtube-dmca]: https://twitter.com/stylewarning/status/1460470872859312133
[^shazam-annoyingly-sparse-paper]: https://www.ee.columbia.edu/~dpwe/papers/Wang03-shazam.pdf
[^heuristic]: https://en.wikipedia.org/wiki/Heuristic
[^phase]: https://en.wikipedia.org/wiki/Phase_(waves)
[^time-freq-analysis]: https://en.wikipedia.org/wiki/Time%E2%80%93frequency_analysis
[^lssa]: https://en.wikipedia.org/wiki/Least-squares_spectral_analysis
[^wavelet]: https://en.wikipedia.org/wiki/Wavelet
[^gsl-wavelets]: https://www.gnu.org/software/gsl/doc/html/dwt.html#wavelet-transforms-in-one-dimension
[^canon-music]: https://en.wikipedia.org/wiki/Canon_(music)
[^convex-function]: https://en.wikipedia.org/wiki/Convex_function
[^gradient-descent]: https://en.wikipedia.org/wiki/Gradient_descent
[^wasm]: https://en.wikipedia.org/wiki/WebAssembly
[^ffmpeg]: https://en.wikipedia.org/wiki/FFmpeg
