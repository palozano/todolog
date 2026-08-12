#!/usr/bin/env ruby
# frozen_string_literal: true

require "fileutils"
require "open3"
require "tmpdir"

ROOT = File.expand_path("..", __dir__)
OUT_DIR = File.join(ROOT, "docs", "screenshots")
FONT = "/System/Library/Fonts/Monaco.ttf"

COLORS = {
  page: "#0b0d0f",
  bg: "#101214",
  bar: "#24282c",
  panel: "#171a1c",
  panel_alt: "#34383b",
  border: "#5a6068",
  text: "#e8e8ea",
  muted: "#9aa7a0",
  dim: "#62676f",
  cyan: "#7dd3fc",
  green: "#65d58a",
  yellow: "#f5d90a",
  selected_bg: "#8e6f28",
  selected_fg: "#fff4c9",
  orange: "#d99032",
  blue: "#7aa2f7"
}.freeze

class Canvas
  def initialize(width, height)
    @width = width
    @height = height
    @args = [
      "magick",
      "-size",
      "#{width}x#{height}",
      "xc:#{COLORS[:page]}",
      "-font",
      FONT
    ]
  end

  def rect(x, y, width, height, fill:, stroke: nil, radius: 0, stroke_width: 1)
    @args += ["-fill", fill, "-stroke", stroke || "none", "-strokewidth", stroke_width.to_s]
    x2 = x + width
    y2 = y + height
    shape = if radius.positive?
      "roundrectangle #{x},#{y} #{x2},#{y2} #{radius},#{radius}"
    else
      "rectangle #{x},#{y} #{x2},#{y2}"
    end
    @args += ["-draw", shape]
  end

  def line(x1, y1, x2, y2, stroke: COLORS[:border], width: 1)
    @args += ["-fill", "none", "-stroke", stroke, "-strokewidth", width.to_s, "-draw", "line #{x1},#{y1} #{x2},#{y2}"]
  end

  def text(x, y, body, fill: COLORS[:text], size: 22, weight: 400, anchor: :start)
    draw_x = case anchor
    when :middle
      x - approx_width(body, size) / 2
    when :end
      x - approx_width(body, size)
    else
      x
    end
    @args += [
      "-fill",
      fill,
      "-stroke",
      "none",
      "-pointsize",
      size.to_s,
      "-draw",
      "text #{draw_x.round},#{y} '#{escape_draw_text(body)}'"
    ]
  end

  def save(path)
    FileUtils.mkdir_p(File.dirname(path))
    ok = system(*(@args + [path]))
    raise "failed to render #{path}" unless ok
  end

  private

  def approx_width(body, size)
    body.to_s.length * size * 0.62
  end

  def escape_draw_text(body)
    body.to_s.gsub("\\", "\\\\\\").gsub("'", "\\\\'")
  end
end

def run!(*args)
  stdout, stderr, status = Open3.capture3(*args, chdir: ROOT)
  raise "#{args.join(" ")} failed:\n#{stderr}\n#{stdout}" unless status.success?

  stdout
end

def terminal(width, height, title)
  canvas = Canvas.new(width, height)
  canvas.rect(16, 16, width - 32, height - 32, fill: COLORS[:bg], stroke: "#2f3439", radius: 8)
  canvas.rect(16, 16, width - 32, 42, fill: COLORS[:bar], radius: 8)
  canvas.rect(16, 42, width - 32, 16, fill: COLORS[:bar])
  canvas.text(38, 44, "todolog", fill: COLORS[:text], size: 18, weight: 700)
  canvas.text(width - 38, 44, title, fill: COLORS[:muted], size: 15, anchor: :end)
  canvas
end

def render_command(output, path, title:, command:)
  lines = output.lines.map(&:chomp)
  width = 1280
  height = [210 + (lines.length * 30), 430].max
  canvas = terminal(width, height, title)
  canvas.text(42, 96, "$ #{command}", fill: COLORS[:green], size: 22, weight: 700)
  y = 138
  lines.each do |line_text|
    canvas.text(42, y, line_text, fill: COLORS[:text], size: 21)
    y += 30
  end
  canvas.save(path)
end

def render_tui(tasks, path)
  width = 1440
  height = 800
  left_x = 40
  top = 122
  help_top = 672
  list_w = 930
  detail_x = 990
  detail_w = 410
  canvas = terminal(width, height, "interactive TUI")

  canvas.text(width / 2, 88, "todolog  |  #{tasks.length} open  |  0 done", fill: COLORS[:text], size: 22, weight: 700, anchor: :middle)
  canvas.rect(left_x, top, list_w, 520, fill: COLORS[:panel], stroke: COLORS[:border], radius: 10)
  canvas.rect(detail_x, top, detail_w, 520, fill: COLORS[:panel], stroke: COLORS[:border], radius: 10)
  canvas.rect(438, top - 14, 136, 24, fill: COLORS[:bg])
  canvas.rect(1128, top - 14, 156, 24, fill: COLORS[:bg])
  canvas.text(506, top + 4, "open tasks", fill: COLORS[:text], size: 17, weight: 700, anchor: :middle)
  canvas.text(1206, top + 4, "selected task", fill: COLORS[:text], size: 17, weight: 700, anchor: :middle)

  row_y = top + 48
  tasks.each_with_index do |task, index|
    selected = index.zero?
    canvas.rect(left_x + 18, row_y - 25, list_w - 36, 54, fill: COLORS[:selected_bg], radius: 4) if selected
    fg = selected ? COLORS[:selected_fg] : COLORS[:text]
    canvas.text(left_x + 38, row_y, task[:marker].ljust(5), fill: COLORS[:cyan], size: 20, weight: 700)
    canvas.text(left_x + 132, row_y, task[:text], fill: fg, size: 20)
    canvas.text(left_x + 132, row_y + 27, "open", fill: COLORS[:green], size: 16)
    canvas.text(left_x + 206, row_y + 27, task[:id], fill: COLORS[:dim], size: 16)
    canvas.text(left_x + 425, row_y + 27, task[:location], fill: COLORS[:blue], size: 16)
    row_y += 78
  end

  first = tasks.first
  detail_y = top + 60
  [
    ["ID", first[:id], COLORS[:text]],
    ["Status", "open", COLORS[:green]],
    ["Marker", first[:marker], COLORS[:cyan]],
    ["Location", first[:location], COLORS[:blue]]
  ].each do |label, value, color|
    canvas.text(detail_x + 34, detail_y, label.ljust(8), fill: COLORS[:muted], size: 18, weight: 700)
    canvas.text(detail_x + 150, detail_y, value, fill: color, size: 18, weight: label == "ID" ? 700 : 400)
    detail_y += 36
  end
  canvas.text(detail_x + 34, detail_y + 26, first[:text], fill: COLORS[:text], size: 19)

  canvas.line(16, help_top, width - 16, help_top, stroke: COLORS[:border])
  canvas.text(width / 2, help_top + 42, "j/k move   PgUp/PgDn jump   Enter open", fill: COLORS[:text], size: 18, anchor: :middle)
  canvas.text(width / 2, help_top + 76, "d done   o reopen   q/Esc quit", fill: COLORS[:text], size: 18, anchor: :middle)
  canvas.save(path)
end

def render_emacs(tasks, path)
  width = 1440
  height = 520
  canvas = terminal(width, height, "Emacs task buffer")
  canvas.rect(32, 78, width - 64, 42, fill: "#2e3336")
  canvas.text(56, 105, "todolog", fill: COLORS[:text], size: 18, weight: 700)
  canvas.text(250, 105, "All", fill: COLORS[:muted], size: 18)
  canvas.text(340, 105, "(#{tasks.length},0)", fill: COLORS[:muted], size: 18)
  canvas.text(520, 105, "(Dired by name WK)", fill: COLORS[:muted], size: 18)
  canvas.rect(32, 120, width - 64, 40, fill: COLORS[:panel_alt])
  canvas.text(96, 148, "ID", fill: COLORS[:text], size: 20, weight: 700)
  canvas.text(322, 148, "Location", fill: COLORS[:text], size: 20, weight: 700)
  canvas.text(575, 148, "Task", fill: COLORS[:text], size: 20, weight: 700)
  y = 192
  tasks.each_with_index do |task, index|
    canvas.text(62, y, (index + 1).to_s, fill: index.zero? ? COLORS[:yellow] : COLORS[:dim], size: 20)
    canvas.text(96, y, task[:id], fill: COLORS[:muted], size: 20)
    canvas.text(322, y, task[:location], fill: COLORS[:orange], size: 20)
    canvas.text(575, y, task[:text], fill: COLORS[:text], size: 20, weight: 700)
    y += 36
  end
  canvas.save(path)
end

Dir.chdir(ROOT) do
  FileUtils.mkdir_p(OUT_DIR)

  Dir.mktmpdir("todolog-tasks") do |dir|
    task_file = File.join(dir, "TASKS.md")
    scan_output = run!("cargo", "run", "--quiet", "--", "scan", "examples", "--output", task_file)
    list_output = run!("cargo", "run", "--quiet", "--", "list", task_file, "--open")
    quickfix_output = run!("cargo", "run", "--quiet", "--", "list", task_file, "--open", "--quickfix")
    emacs_output = run!("cargo", "run", "--quiet", "--", "list", task_file, "--open", "--emacs")

    render_command(scan_output, File.join(OUT_DIR, "todolog-scan.png"), title: "scan", command: "todolog scan examples")
    render_command(list_output, File.join(OUT_DIR, "todolog-list.png"), title: "list output", command: "todolog list --open")
    render_command(quickfix_output, File.join(OUT_DIR, "todolog-quickfix.png"), title: "quickfix output", command: "todolog list --open --quickfix")

    tasks = emacs_output.lines.map do |line|
      line = line.chomp
      match = line.match(/\A(?<location>.+:\d+):1: (?<text>.*) \[(?<id>[^\]]+)\]\z/)
      next unless match

      {
        id: match[:id],
        marker: "TODO",
        location: match[:location],
        text: match[:text]
      }
    end.compact

    render_tui(tasks, File.join(OUT_DIR, "todolog-tui.png"))
    render_emacs(tasks, File.join(OUT_DIR, "todolog-emacs.png"))
  end
end

puts "wrote screenshots to #{OUT_DIR}"
