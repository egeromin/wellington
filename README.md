# wellington


Wellington is a lightweight static blog generator written in Rust. You can see
it in action at https://emanuelgeromin.com/blog/

Wellington allows you to write blog posts in markdown and then automatically

- renders them as HTML
- updates the table of contents
- updates the RSS feed.

From the root directory of your website, create a new blog:


```bash
mkdir my-amazing-blog
cd my-amazing-blog
wellington init --title "My Blog" \
    --home_url "https://mysite.com \
    --desc "My blog about clogs" \
    --author "It's Me"
```


Then write a blog post:


```bash
mkdir post-1
touch post-1/index.md  # edit this file
wellington sync
```

This last command, `wellington sync`, automatically 

- renders the post as HTML at post-1/index.html
- creates/updates the table of contents at index.html
- creates/updates the rss feed at rss.xml


## Installation

Installation is via cargo:

```bash
cargo install wellington
```


## Markdown with Footnotes/Sidenotes

Wellington supports an extended markdown syntax. In addition to the usual
markdown, you can also mark footnotes/sidenotes directly in the HTML using
curly braces:

```markdown

# Here is an article

Here is some text with sidenotes/footnotes{this text will render as a
footnote/sidenote}

```

**Sidenotes** are like footnotes except that they display to the right of the
article instead of the bottom. See [this
blogpost](https://emanuelgeromin.com/blog/intro-reinforcement-learning-tic-tac-toe/)
as an example. To work, they require proper CSS. See the 'CSS'
section below.





## Why Wellington

There are plenty of blogging engines out there, many designed for use with
static hosting services like GitHub pages. One great example is
[Jekyll](https://jekyllrb.com/).

I wrote wellington because I wanted support for sidenotes and require only a
very simple set of features.


## Relative links

Relative links in the post markdown files are automatically changed to point to the directory they're in. So for example, if `my-amazing-blog/post-1/index.md` contains the following:

```markdown
![an image](image.png)
```

Then this is rendered as:

```html
<img src="my-amazing-blog/post-1/image.png" />
```

This makes embedding images in the blog post easier.


## CSS 

Wellington's sidenotes were designed for use with
[tufte-css](https://github.com/edwardtufte/tufte-css). I use a modified version on [my blog](https://emanuelgeromin.com/blog/tufte.css). This modified version is more mobile friendly and on narrow screens, instead of displaying sidenotes, displays footnotes. You're very welcome to copy my CSS. Alternatively, as a more minimal solution, you can just hide the sidenotes if you don't require them and render only the footnotes:

```css
.sidenote {
    display: none;
}
```

To display the blog posts in a mobile friendly way, by using sidenotes for desktop and footnotes for mobile, as I've done, please
check the [modified version of tufte-css](https://emanuelgeromin.com/blog/tufte.css).


## Templates

To render the posts and the table of contents, wellington uses
[Handlebars](https://github.com/sunng87/handlebars-rust), a templating engine.
This allows for customisability of the posts and of the table of contents.
There are default templates in the source code in the `templates` directory.
These are used by default and do not require installation. To override them,
provide your own templates in your blog home directory:

- `.index_template.html` for the table of contents
- `.post_template.html` for the posts

Take a look at the default templates and adapt them to suit your needs!


## MathJax (Latex) support

Using mathjax (latex) in the blog posts requires a hack. To prevent the latex
to be rendered by the markdown parser, it has to be marked as code in markdown,
for example

```markdown

# a markdown file

Here is some text with an equation: `\(E = mc^2\)`

```

By default, mathjax does not render anything wrapped in `<pre>`
or `<code>` tags. To override this behaviour, 
in addition to the `<script>` tag linking to the mathjax CDN, you will have to add
the following to your template `<head>` section:

```html
<script type="text/x-mathjax-config">
MathJax.Hub.Config({
  tex2jax: {
      skipTags: ["script","noscript","style","textarea"]
      }
  });
</script>
```

This overrides the default `skipTags`, allowing also latex in `<pre>` and
`<code>` tags to be rendered.

If you require both rendered latex and latex code in your blog posts, then
unfortunately at the moment you're out of luck. A more long term solution would
be to parse latex properly in the markdown document. However, this is a more
long term project as it requires modifying `pulldown-cmark`, the library I use
for rendering markdown. Contributions are welcome!


## Known Bugs

- Footnotes are not rendered properly as all inline markdown formatting is lost
  (#3)


## Roadmap

There is still much to do. The current codebase grew organically without much
advance planning. This shows: the codebase is quite messy and in need of a
refactor!

Feature wise, soon I hope to support 

- syntax highlighting (#1)
- make the template variables `post_url` and `index_url` available in the
  markdown as well (#5). This is useful when embedding external code or demos in the
  blog post.

