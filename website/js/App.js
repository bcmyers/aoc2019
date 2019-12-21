import React from 'react'

const App = props => {
  var width = window.innerWidth
  var height = window.innerHeight
  if (props && props.width && props.height) {
    width = props.width
    height = props.height
  }

  const canvasRef = React.useRef(null)

  React.useEffect(() => {
    const can = canvasRef.current
    Game.new(can).then(game => {
      game.run()
    })
  })

  return <canvas ref={canvasRef} width={width} height={height} />
}

class Game {
  static async new(can) {
    const rust = await import('../pkg/index')
    const wasm = await import('../pkg/index_bg')
    return new Game(can, rust, wasm)
  }

  constructor(can, rust, wasm) {
    this.game = new rust.Game()
    this.canvas = new Canvas(can, this.game.cols(), this.game.rows())
    this.wasm = wasm
  }

  run() {
    const next_move = this.game.step()
    if (next_move !== undefined) {
      this.render()
      this.game.input(next_move)
      setTimeout(this.run.bind(this), 3)
    }
  }

  render() {
    this.canvas.clear()
    this.canvas.drawBorder()
    this.canvas.drawScore(this.game.score())
    const display = new Uint8Array(
      this.wasm.memory.buffer,
      this.game.display(),
      this.game.display_len(),
    )
    const cols = this.game.cols()
    display.forEach((byte, i) => {
      let y = Math.floor(i / cols)
      let x = i % cols
      if (byte === 0) {
      } else if (byte === 1) {
        this.canvas.drawBlock(x, y, 'black')
      } else if (byte === 2) {
        this.canvas.drawBlock(x, y, 'navy')
      } else if (byte === 3) {
        this.canvas.drawBlock(x, y, 'navy')
      } else if (byte === 4) {
        this.canvas.drawBall(x, y, 'red')
      }
    })
  }
}

class Canvas {
  constructor(can, cols, rows) {
    this.can = can
    this.ctx = can.getContext('2d')
    this.cols = cols
    this.rows = rows
    this.width = can.width
    this.height = can.height
    this.colWidth = can.width / cols
    this.rowHeight = can.height / rows
  }

  clear() {
    this.ctx.clearRect(0, 0, this.width, this.height)
  }

  drawBorder() {
    this.ctx.globalCompositeOperation = 'source-over'
    this.ctx.lineWidth = 5
    this.ctx.strokeStyle = '#000000'
    this.ctx.strokeRect(0, 0, this.width, this.height)
  }

  drawBall(x, y, color) {
    if (x > this.cols || y > this.rows) {
      throw new Error(`(${x}, ${y}) is out of bounds.`)
    }
    if (!color) {
      color = 'navy'
    }
    x = this.colWidth * (x + 0.5)
    y = this.rowHeight * (y + 0.5)
    this.ctx.beginPath()
    this.ctx.arc(x, y, this.rowHeight / 2, 0, 2 * Math.PI, false)
    this.ctx.fillStyle = color
    this.ctx.fill()
  }

  drawBlock(x, y, color) {
    if (x > this.cols || y > this.rows) {
      throw new Error(`(${x}, ${y}) is out of bounds.`)
    }
    if (!color) {
      color = 'navy'
    }
    x = this.colWidth * x
    y = this.rowHeight * y
    this.ctx.beginPath()
    this.ctx.rect(x, y, this.colWidth, this.rowHeight)
    this.ctx.fillStyle = color
    this.ctx.fill()
  }

  drawScore(score) {
    this.ctx.font = '25px Arial'
    this.ctx.fillStyle = '0,0,0'
    this.ctx.fillText(
      score,
      (this.cols - 3) * this.colWidth,
      1.75 * this.rowHeight,
    )
  }
}

export default App
