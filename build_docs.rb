#!/usr/bin/env ruby

require 'find'

unless system("cargo", "doc", "--no-deps", "--bin", "empowerd")
  puts "Building docs failed!"
  exit 1
end

bad_files = [
  "constant.BRANCHES.html",
  "constant.COUNT.html",
  "static.LOC.html",
  "static.RS.html",
]

Find.find("#{__dir__}/target/doc") do |path|
  next unless File.file? path

  next File.delete(path) if bad_files.include? File.basename(path)
  ext = File.extname(path)
  next unless [".html", ".js"].include? ext

  content = File.read(path)
  if ext == ".html"
    content.gsub! %r{
      <li>\s*
        <a\s+href="(?:[a-zA-Z0-9_]+?/)*?static\.(?:LOC|RS)\.html">
          (?:[a-zA-Z0-9_]+?::)*?(?:LOC|RS)
        </a>\s*
      </li>
    }x, ""
    content.gsub! %r{
      <li>\s*
        <a\s+href="(?:[a-zA-Z0-9_]+?/)*?constant\.(?:COUNT|BRANCHES)\.html">
          (?:[a-zA-Z0-9_]+?::)*?(?:COUNT|BRANCHES)
        </a>\s*
      </li>
    }x, ""
    content.gsub! %r{
      <li>\s*
        <div\s+class="item-name">\s*
          <a\s+class="static"\s+href="static\.(?:LOC|RS)\.html"\s+
              title="static\s+(?:[a-zA-Z0-9_]+?::)*?(?:LOC|RS)">
            (?:LOC|RS)
          </a>\s*
          <span\s+title="Restricted\s+Visibility">&nbsp;🔒</span>\s*
        </div>\s*
      </li>
    }x, ""
    content.gsub! %r{
      <li>\s*
        <div\s+class="item-name">\s*
          <a\s+class="constant"\s+href="constant\.(?:COUNT|BRANCHES)\.html"\s+
              title="constant\s+(?:[a-zA-Z0-9_]+?::)*?(?:COUNT|BRANCHES)">
            (?:COUNT|BRANCHES)
          </a>\s*
          <span\s+title="Restricted\s+Visibility">&nbsp;🔒</span>\s*
        </div>\s*
      </li>
    }x, ""
  elsif ext == ".js"
    content.gsub! %r{
      "static:"\["LOC","RS"\],
    }x, ""
    content.gsub! %r{
      "constant":\[(?:"BRANCHES")?,?(?:"COUNT")?\],
    }x, ""
    content.gsub! %r{
      "(?:LOC|RS|COUNT|BRANCHES)",
    }x, ""
  end

  File.write(path, content)
end
