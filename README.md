# Stuart Downing Portfolio


<br>
<div style="display: flex;
  justify-content: center;
  align-items: center;
background-color: #0000ff25;">
<img width="192" src=".github/assets/cool_s.svg" alt="">
</div>

[![Build Status](https://github.com/NtLoadDriverEx/Portfolio/workflows/CI/badge.svg)](https://github.com/NtLoadDriverEx/Portfolio/actions?workflow=CI)

## How?
This project is based on [eframe_template](https://github.com/emilk/eframe_template/) built in Rust targeting wasm to run
in the browser.
## Goals
- [x] Write a better readme
- [ ] Expand each set of windows into their own components (make `.rs` files for each 'page')
- [ ] Make a stock / trading view component
- [ ] Make a pretty component with graphics and FFT generated audio for rain noises
- [ ] Make a mini-game component complete with audio and enjoyable (and simple) gameplay in 2d
### Project Structure
`src/app.rs` conatins the main page layout as you would expect in any web project written in Javascript for example.
`assets/text_contents.toml` contains each widgets large text content. For example if you have a large EasyMark document 
you don't want that stored in your `.rs` file as that would bloat your code.
 
### Running Natively

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Web Locally

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

We use [Trunk](https://trunkrs.dev/) to build for web target.
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> `assets/sw.js` script will try to cache our app, and loads the cached version when it cannot connect to server allowing your app to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing us to load the latest builds during development.

### Web Deploy
1. Just run `trunk build --release`.
2. It will generate a `dist` directory as a "static html" website
3. Upload the `dist` directory to any of the numerous free hosting websites including [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
4. we already provide a workflow that auto-deploys our app to GitHub pages if you enable it.
> To enable Github Pages, you need to go to Repository -> Settings -> Pages -> Source -> set to `gh-pages` branch and `/` (root).
>
> If `gh-pages` is not available in `Source`, just create and push a branch called `gh-pages` and it should be available.

You can test the template app at <https://emilk.github.io/eframe_template/>.