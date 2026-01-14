'use client'

import { useEffect, useState } from 'react'
import type { Room } from '@/lib/types'
import { useSocket } from '@/hooks/useSocket'

interface RoomListProps {
  onJoinRoom?: (roomId: string) => void
}

export default function RoomList({ onJoinRoom }: RoomListProps) {
  const [rooms, setRooms] = useState<Room[]>([])
  const { isConnected, emit, on, off } = useSocket()

  useEffect(() => {
    if (!isConnected) return

    // Request room list
    emit('getRooms')

    // Listen for room list updates
    const handleRoomList = (updatedRooms: Room[]) => {
      setRooms(updatedRooms)
    }

    on('roomList', handleRoomList)

    return () => {
      off('roomList', handleRoomList)
    }
  }, [isConnected, emit, on, off])

  const handleJoin = (roomId: string) => {
    if (onJoinRoom) {
      onJoinRoom(roomId)
    }
  }

  return (
    <div>
      {!isConnected && (
        <div className="text-center text-muted pulse" style={{ padding: 'var(--spacing-lg)' }}>
          サーバーに接続中...
        </div>
      )}

      {isConnected && rooms.length === 0 && (
        <div className="text-center text-muted" style={{ padding: 'var(--spacing-2xl)' }}>
          <p>現在、利用可能なルームはありません。</p>
          <p style={{ marginTop: 'var(--spacing-sm)' }}>
            新しいルームを作成して対戦を開始しましょう！
          </p>
        </div>
      )}

      {isConnected && rooms.length > 0 && (
        <div style={{ display: 'grid', gap: 'var(--spacing-md)' }}>
          {rooms.map((room) => (
            <div
              key={room.id}
              className="card fade-in"
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                padding: 'var(--spacing-md)',
                cursor: 'pointer',
                transition: 'transform var(--transition-fast)',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.transform = 'translateX(4px)'
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.transform = 'translateX(0)'
              }}
            >
              <div>
                <h3 style={{ fontWeight: '600', marginBottom: 'var(--spacing-xs)' }}>
                  {room.name}
                </h3>
                <div
                  style={{
                    display: 'flex',
                    gap: 'var(--spacing-md)',
                    fontSize: 'var(--font-size-sm)',
                    color: 'var(--color-text-light)',
                  }}
                >
                  <span>盤: {room.boardType}</span>
                  <span>
                    プレイヤー: {[room.player1Id, room.player2Id].filter(Boolean).length}/2
                  </span>
                  <span>
                    状態:{' '}
                    {room.status === 'waiting'
                      ? '待機中'
                      : room.status === 'playing'
                        ? 'プレイ中'
                        : '終了'}
                  </span>
                </div>
              </div>
              <button
                className="btn btn-outline"
                onClick={() => handleJoin(room.id)}
                disabled={room.status !== 'waiting'}
              >
                {room.status === 'waiting' ? '参加' : 'プレイ中'}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
