import './style.css';

declare const __SDS_BUILD_ID__: string;

document.querySelectorAll<HTMLElement>('[data-build-id]').forEach((element) => {
  element.textContent = __SDS_BUILD_ID__;
});

if ('serviceWorker' in navigator && location.protocol !== 'file:') {
  window.addEventListener('load', () => { void navigator.serviceWorker.register('/sw.js'); });
}
