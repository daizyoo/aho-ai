'use client'

import { useEffect, useState } from 'react'
import { useParams } from 'next/navigation'
import { useSocket } from '@/hooks/useSocket'
import { createInitialBoard } from '@/lib/game/board'
import type { GameState, Position, Move } from '@/lib/types'
import Board from '@/components/Board'
import HandPieces from '@/components/HandPieces'

export default function RoomPage() {
  const params = useParams()
  const roomId = params.id as string
  const [gameState, setGameState] = useState<GameState | null>(null)
  const { isConnected, emit, on, off } = useSocket()

  useEffect(() => {
    if (!isConnected) return

    // Join the room
    emit('joinRoom', roomId)

    // Initialize game state if not exists
    if (!gameState) {
      const initialBoard = createInitialBoard('shogi')
      setGameState({
        board: initialBoard,
        hands: { 1: {}, 2: {} },
        currentTurn: 1,
        moves: [],
        status: 'playing',
      })
    }

    // Listen for game state updates
    const handleGameStateUpdate = (newState: GameState) => {
      setGameState(newState)
    }

    const handleGameOver = (winner: 1 | 2 | 'draw') => {
      alert(
        winner === 'draw' ? '引き分けです！' : `プレイヤー ${winner} の勝利！`
      )
    }

    on('gameStateUpdate', handleGameStateUpdate)
    on('gameOver', handleGameOver)

    return () => {
      emit('leaveRoom', roomId)
      off('gameStateUpdate', handleGameStateUpdate)
      off('gameOver', handleGameOver)
    }
  }, [isConnected, roomId])

  const handleMove = (from: Position, to: Position) => {
    if (!gameState) return

    const piece = gameState.board[from.row][from.col]
    if (!piece) return

    const move: Move = {
      from,
      to,
      piece,
    }

    emit('makeMove', roomId, move)
  }

  if (!gameState) {
    return (
      <div className="container text-center" style={{ paddingTop: '2rem' }}>
        <div className="pulse">読み込み中...</div>
      </div>
    )
  }

  return (
    <main className="container" style={{ paddingTop: '2rem', paddingBottom: '2rem' }}>
      <h1
        style={{
          fontSize: 'var(--font-size-2xl)',
          fontWeight: 'bold',
          marginBottom: 'var(--spacing-lg)',
          textAlign: 'center',
        }}
      >
        ゲームルーム
      </h1>

      <div
        style={{
          display: 'flex',
          gap: 'var(--spacing-xl)',
          justifyContent: 'center',
          alignItems: 'flex-start',
          flexWrap: 'wrap',
        }}
      >
        {/* Player 2's hand pieces */}
        <HandPieces hand={gameState.hands[2]} playerName="相手の持ち駒" />

        {/* Board */}
        <Board
          board={gameState.board}
          currentPlayer={gameState.currentTurn}
          onMove={handleMove}
        />

        {/* Player 1's hand pieces */}
        <HandPieces hand={gameState.hands[1]} playerName="あなたの持ち駒" />
      </div>

      {/* Game info */}
      <div className="card text-center mt-lg">
        <p style={{ fontSize: 'var(--font-size-lg)', fontWeight: '600' }}>
          現在のターン: プレイヤー {gameState.currentTurn}
        </p>
        <p className="text-muted mt-sm">
          {gameState.currentTurn === 1 ? 'あなたのターン' : '相手のターン'}
        </p>
      </div>

      {/* Back button */}
      <div className="text-center mt-lg">
        <button className="btn btn-secondary" onClick={() => (window.location.href = '/')}>
          ルーム一覧に戻る
        </button>
      </div>
    </main>
  )
}
