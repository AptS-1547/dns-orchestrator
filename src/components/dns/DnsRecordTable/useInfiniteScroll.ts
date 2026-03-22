import { useCallback, useEffect, useRef } from "react"

interface UseInfiniteScrollOptions {
  hasMore: boolean
  isLoadingMore: boolean
  enabled: boolean
  onLoadMore: () => void
  rootMargin?: string
}

export function useInfiniteScroll({
  hasMore,
  isLoadingMore,
  enabled,
  onLoadMore,
  rootMargin = "100px",
}: UseInfiniteScrollOptions) {
  const sentinelRef = useRef<HTMLElement | null>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  const setSentinelRef = useCallback((node: HTMLElement | null) => {
    sentinelRef.current = node
  }, [])

  const handleObserver = useCallback(
    (entries: IntersectionObserverEntry[]) => {
      const [entry] = entries
      if (entry.isIntersecting && hasMore && !isLoadingMore && enabled) {
        onLoadMore()
      }
    },
    [hasMore, isLoadingMore, onLoadMore, enabled]
  )

  useEffect(() => {
    if (!enabled) return

    const sentinel = sentinelRef.current
    const scrollContainer = scrollContainerRef.current
    if (!(sentinel && scrollContainer)) return

    const observer = new IntersectionObserver(handleObserver, {
      root: scrollContainer,
      rootMargin,
    })
    observer.observe(sentinel)

    return () => observer.disconnect()
  }, [handleObserver, enabled, rootMargin])

  return { scrollContainerRef, setSentinelRef }
}
