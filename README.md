# Bevy Hyda
## Run HTML (&amp; soon CSS) in the Bevy game engine!

## WARNING:
- This is 0.1.0, meaning that it can render basic HTML 4.0, its not prepared for websites that are heavily stylized.
- I've made this in two weeks in my free time for fun.
- Source code is probably ***UGLY***. (Feel free to send PRs or contribute to make the code much better :D).
- I'm not an expert using GitHub and this is my first "crate" ;_;, so please feel free to give any advice or point out if i'm doing something wrong in this repository.

## Introduction.
Bevy Hyda is a system that allows you to load HTML sites in your Bevy app/game. It is fully powered by BevyUI, so you can access, modify and use the elements like you do usually in Bevy! (without any 'but's).

## Websites running with Bevy Hyda.
- Acid Test 1 (no custom CSS): ![image](https://github.com/user-attachments/assets/0a14de56-12ef-4e52-9000-f31e9f620428)
- The first WWW website: ![image](https://github.com/user-attachments/assets/2afc2f0e-0667-41eb-a278-a8ec7434f153)

## Installation.
- First, make sure that you're using Bevy 0.14.0 (and up) in your project.
- Clone the repository: `git clone https://github.com/TheNachoBIT/bevy_hyda`
- Open your project's Cargo.toml and add it in dependencies:
  
  ```toml
  # ...
  [dependencies]
  bevy = "0.14.0"
  bevy_hyda = { path = "path/to/bevy_hyda" }
  # ...
  ```
- Done! Now you can build/run your project to see if everything's up and running as it should!

## How to use it?
- First, in your code, you add the `HydaBevyPlugin` to your Bevy app with `add_plugins()`.
- After that, inside of a setup system that contains a `mut Commands` and a `Res<AssetServer>` (could've be an existing one or a new one, its up to you :D), initialize a `HydaComponent` by opening an HTML file with `html_file()` or use a string with `html_string()`:
  ```rs
  let get_html = bevy_hyda::html_file("assets/path/to.html".to_string());
  ```
  ```rs
  let get_html = bevy_hyda::html_string(r#"
  <html>
    <body>
      <h1>Hello World!</h1>
    </body>
  </html>
  "#.to_string());
  ```
- Finally, we spawn the UI with `spawn_ui()`, passing the commands and asset server to the function:
  ```rs
  get_html.spawn_ui(&mut commands, &asset_server);
  ```
- And you're done! You can just build & run your project and you're going to see your wonderful HTML file displayed in Bevy! :D

  If you want to see the full code, you can check out the examples folder in this repo!
