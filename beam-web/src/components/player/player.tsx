import '@vidstack/react/player/styles/base.css';

import { useEffect, useRef } from 'react';

import {
  isHLSProvider,
  MediaPlayer,
  MediaProvider,
  Poster,
  Track,
  type TrackProps,
  type MediaCanPlayDetail,
  type MediaCanPlayEvent,
  type MediaPlayerInstance,
  type MediaProviderAdapter,
  type MediaProviderChangeEvent,
} from '@vidstack/react';

import { VideoLayout } from './layouts/video-layout';

export interface PlayerProps {
  src: string;
  posterSrc: string;
  posterAlt: string;
  title: string;
  thumbnails: string;
  textTracks: readonly TrackProps[];
  // crossOrigin: boolean;
  // playsInline: boolean;
  // onProviderChange: (provider: MediaProviderAdapter | null, nativeEvent: MediaProviderChangeEvent) => void;
  // onCanPlay: (detail: MediaCanPlayDetail, nativeEvent: MediaCanPlayEvent) => void;
  // ref: React.Ref<MediaPlayerInstance>;
}

const Player = (props: PlayerProps) => {
  let player = useRef<MediaPlayerInstance>(null);

  // TODO: See if needed
  useEffect(() => {
    // Subscribe to state updates.
    return player.current!.subscribe(({ paused, viewType }) => {
      // console.log('is paused?', '->', state.paused);
      // console.log('is audio view?', '->', state.viewType === 'audio');
    });
  }, []);

  // TODO: See if needed
  function onProviderChange(
    provider: MediaProviderAdapter | null,
    nativeEvent: MediaProviderChangeEvent,
  ) {
    // We can configure provider's here.
    if (isHLSProvider(provider)) {
      provider.config = {};
    }
  }

  // TODO: See if needed
  // We can listen for the `can-play` event to be notified when the player is ready.
  function onCanPlay(detail: MediaCanPlayDetail, nativeEvent: MediaCanPlayEvent) {
    // ...
  } 

  return (
    <MediaPlayer
      className="w-full aspect-video bg-black text-white font-sans overflow-hidden rounded-md ring-media-focus data-[focus]:ring-4"
      title="Sprite Fight"
      src={props.src}
      crossOrigin
      playsInline
      onProviderChange={onProviderChange}
      onCanPlay={onCanPlay}
      ref={player}
    >
      <MediaProvider>
        <Poster
          className="absolute inset-0 block h-full w-full rounded-md opacity-0 transition-opacity data-[visible]:opacity-100 object-cover"
          src={props.posterSrc}
          alt={props.posterAlt}
        />
        {props.textTracks.map((track) => (
          <Track {...track} key={track.src} />
        ))}
      </MediaProvider>

      <VideoLayout thumbnails={props.thumbnails} />
    </MediaPlayer>
  );
};
// TODO: Implement multiple resolution, speed control, display buffer progress, and more
// TODO: Add statistics menu to cover
// TODO: Add prop to support previious and next track buttons

export default Player;
