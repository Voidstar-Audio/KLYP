#let os = sys.inputs.os;

#set text(font: "Archivo", size: 11pt)
#set page(
  fill: rgb("DBDDE5"),
  margin: 5em,
  paper: "a4",
)
#show heading: set text(weight: "medium", size: 1.5em, tracking: -0.035em)
#show heading: set block(below: 1em)
#set par(leading: 0.8em, spacing: 1.5em)

#let hint(body) = {
  rect(
    width: 100%,
    fill: rgb("2079D122"),
    inset: 1em,
    [
      #text([Hint], weight: "medium", fill: rgb("2079D1"))\
      #body
    ]
  )
}

#let note(body) = {
  rect(
    width: 100%,
    fill: rgb("D85C0022"),
    inset: 1em,
    [
      #text([Note], weight: "medium", fill: rgb("D85C00"))\
      #body
    ]
  )
}

#page(
  [
    #set text(fill: rgb("C0C3CC"))
    #par(
      text(
        [KLYP\ User Manual#h(1fr) ],
        size: 1.5*1.5*1.5*1.5em,
        weight: "bold",
        tracking: -0.05em,
      ),
      leading: 0.25em
    )
    Version 1.0.1 for #if os == "macos" [macOS] else if os == "windows" [Windows] else if os == "linux" [Linux]

    #align(bottom + right)[
      #box(image("logo_dark.svg", height: 48pt))
    ]
  ],
  fill: rgb("000000")
)

#pagebreak()

#set page(footer: context [
  KLYP User Manual #h(1fr) #counter(page).display()
])

Thank you for downloading KLYP!\
This manual will guide you through its installation and usage.

#h(1fr)

#outline(title: none)

#pagebreak()

= Installation

#if os == "windows" [
  To install KLYP as a VST3 plugin, move `Klyp.vst3` into your VST3 folder.\ 
  Typically, this is `C:\Program Files\Common Files\VST3`

  #align(center)[#image("windows-install-vst3.png")]

  To get to this folder in Explorer, press `ctrl+L`.\ 
  Then, type in `C:\Program Files\Common Files\VST3` and press enter.

  #pagebreak()

  You can also install KLYP as a CLAP by copying it to your CLAP folder.

  #hint[
    You don't need to install KLYP as a CLAP if you have already installed it as a VST3.

    CLAP is a new standard for audio plugins.
    It brings better performance and a lot of improvements over standards like VST3.
    However, only some DAWs, such as Bitwig Studio and REAPER currently support it.
    
    If you're interested, you can learn more about CLAP here:\
    #underline(link("https://u-he.com/community/clap/")[CLAP: The New Audio Plug-in Standard])
  ]

  To install the CLAP, move `Klyp.clap` into your CLAP folder.\ 
  Typically, this is `C:\Program Files\Common Files\CLAP`

  #align(center)[#image("windows-install-clap.png")]
] else if os == "macos" [
  To install KLYP as a VST3 plugin, copy it to your VST3 folder.
  Typically, this folder is located at:\ 
  #align(center)[`/Library/Audio/Plug-ins/VST3`]

  To get to this folder in Finder, you can choose Go > Go to Folder from the Menu Bar.
  Then, type in the pathname and select the matching folder.

  #image("macos-go-to.png", height: 300pt)

  Then, just move the .vst3 file into this folder.

  #align(center)[#image("macos-install-vst3.png", height: 260pt)]

  #pagebreak()

  You can also install KLYP as a CLAP by copying it to your CLAP folder.
  
  #hint[
    You don't need to install KLYP as a CLAP if you have already installed it as a VST3.

    CLAP is a new standard for audio plugins.
    It brings better performance and a lot of improvements over standards like VST3.
    However, only some DAWs, such as Bitwig Studio and REAPER currently support it.
    
    If you're interested, you can learn more about CLAP here:\
    #underline(link("https://u-he.com/community/clap/")[CLAP: The New Audio Plug-in Standard])
  ]

  To install the CLAP, move `Klyp.clap` into your CLAP folder.\ 
  Typically, this is `/Library/Audio/Plug-ins/CLAP`

  #align(center)[#image("macos-install-clap.png", height: 260pt)]
] else if os == "linux" [
  To install the VST3, move `Klyp.vst3` into your VST3 folder.\
  Choose `~/.vst3` to install the plugin for your user.\
  Choose `/usr/lib/vst3` for system-wide installation.

  #align(center)[#image("linux-install-vst3.png", height: 260pt)]

  To install the CLAP, move `Klyp.clap` into your CLAP folder.\
  Choose `~/.clap` to install the plugin for your user.\
  Choose `/usr/lib/clap` for system-wide installation.

  #align(center)[#image("linux-install-clap.png", height: 260pt)]
]

#pagebreak()

= Overview

KLYP's user interface is streamlined, easy to use, and takes an interactive approach to soft clipping.
It provides visualization to help you see what effect the plugin has on your audio.

// TODO Image

At a glance, KLYP's interface mainly consists of:

#grid(
  columns:2,
  column-gutter: 1em,
  row-gutter: 0.75em,
  [*Controls*],              [Sliders that affect KLYP's clipping curve.],
  [*Antialiasing Settings*], [Menu to apply settings that reduce aliasing.],
  [*Visualizers*],           [Cohesive, rich visualizers that are interactive.],
)

The following sections go over each of these in detail.

#pagebreak()

= Controls

The sliders in this section affect KLYP's clipping curve.

// TODO Image

#grid(
  columns:2,
  column-gutter: 1em,
  row-gutter: 0.75em,
  [*`PRE-GAIN`*],  [Boosts or attenuates the incoming audio.],
  [*`SOFTNESS`*],  [Interpolates the clipping curve between hard and soft.\ The higher the softness, the less harsh the distortion.],
  [*`THRESHOLD`*], [Changes at what level audio starts to clip.],
)

// TODO Image of single slider

Drag on a slider to change its value.\
Drag on it while holding the `shift` key to fine-tune its value.\
Click on it while holding the #if os == "macos" [`cmd`] else [`ctrl`] key to reset it to its default value.\
Double-click it to type in its value.\

#pagebreak()

= Antialiasing Settings

Antialiasing reduces aliases, which are unwanted frequencies that ocurr as a natural consequence of digital distortion.

KLYP uses linear-phase oversampling to antialias while eliminating cramping in the upper frequencies.
High factors of oversampling increases CPU use and leads to increased latency as well as subtle pre-ringing.
KLYP also provides antiderivative antialiasing for a lower-cost improvement on top of oversampling.

Select the Antialiasing dropdown to access these settings.

// TODO Image

#grid(
  columns:2,
  column-gutter: 1em,
  row-gutter: 0.75em,
  [*`OVERSAMPLING`*],   [Controls the factor by which audio is oversampled.\ For instance, when `2x` oversampling is enabled in a host with a 44.1 kHz sample rate, audio is processed at 88.2 kHz.],
  [*`ANTIDERIVATIVE`*], [Controls whether antiderivative antialiasing is used.],
)

#pagebreak()

= Visualizers

// TODO Image where all visualizers are highlighted

Visualizers are KLYP's superpower.
They are not just information-rich and interactive, but they play together cohesively.
The following sections go over each visualizer in detail.

== Clipping Curve

// TODO Image of clipping curve

The clipping curve plots out the distortion that KLYP applies.

The X axis corresponds to the input level.\
The Y axis corresponds to the output level.

As audio runs through KLYP, this curve behaves like a peak meter.
The filled in portion of it visualizes the input level.

== Oscilloscope

// TODO Image of oscope

The oscilloscope shows the waveform of the output audio.
Behind it, the input audio waveform is faintly visible.

== Threshold Line

// TODO Image

The threshold line visualizes the audio level where clipping takes place.

The thin red line is the threshold, the point of full saturation of clipping.
It directly corresponds to the `THRESHOLD` parameter.

The filled in portion below it is the non-linear area.
It grows when the `SOFTNESS` parameter is turned up.
Within this area, audio is distorted, but not directly clipped.

// TODO Image

All of KLYP's visualizers are scaled to each other.
This means, for instance, that the threshold line applies to the clipping curve, as well as the oscilloscope.

#pagebreak()

= Changelog

== Version 1.0.1

// TODO

== Version 1.0.0

Initial release.

#page(
  footer: none,
  fill: rgb("000000"),
  align(bottom + right, link("https://voidstaraudio.com", image("logo_dark.svg", height: 48pt)))
)

