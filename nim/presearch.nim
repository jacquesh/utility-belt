import os
import strutils
import terminal

proc offset_of_first_matching_line(inputFile: File, inputLength: BiggestInt, prefix: string): BiggestInt =
  var # These bounds always point to the start  of a line (conceptually, the upper bound initially points to the byte after the last byte in the file, which would be the "start of a line")
    lowerBoundByte: BiggestInt = 0
    upperBoundByte = inputLength

  # First phase: Binary search to narrow down our window to a small number of lines
  while lowerBoundByte < upperBoundByte:
    let midpointByte = lowerBoundByte + ((upperBoundByte - lowerBoundByte) div 2)
    inputFile.setFilePos(midpointByte)
    discard inputFile.readLine() # Discard the remainder of the line we were on, since we almost certainly were in the middle of a line

    if inputFile.endOfFile():
      # If our midpoint is on the last line of the file, then break and continue straight to the second phase
      break

    let nextLineStartByte = inputFile.getFilePos()
    let nextLine = inputFile.readLine()
    let nextLineEndByte = inputFile.getFilePos()

    if nextLineEndByte >= upperBoundByte:
      # If the line we're about to compare is the last line of our window then break to the second phase.
      # If we didn't do this, and our window consisted of 3 lines of equal length in the middle of the file,
      # then we'd get a midpoint in the middle of the second line, discard the rest of the second line and
      # read the third line.
      # If that third line matched the desired prefix, the window's upper bound would be set to the end of
      # that line and therefore would not change, so we'd get stuck in an infinite loop.
      break

    let compareLen = min(len(nextLine), len(prefix))
    let prefixCompare = prefix[0..compareLen-1]
    let nextLineCompare = nextLine[0..compareLen-1]
    let compareResult = cmp(prefixCompare, nextLineCompare)

    if compareResult < 0:
      # The prefix is earlier in the file than the comparison line, so bring down the upper bound
      # to the beginning of the comparison line, excluding it from the window
      upperBoundByte = nextLineStartByte

    elif compareResult > 0:
      # The prefix is later in the file than the comparison line, so push the lower bound up
      # to the end of the comparison line (or rather the start of the following line),
      # excluding it from the window
      lowerBoundByte = nextLineEndByte

    else: # compareResult == 0
      # The comparison line has the required prefix. We're looking for the *first* instance of
      # the prefix so bring the upper bound down to the end of the comparison line.
      # Our window now still includes the comparison line, but nothing after it.
      # We know that this will still make the window at least 1 byte smaller, because
      # we checked above to see that the end of the comparison line is earlier in the file
      # than our upper bound byte (or symbolically: nextLineEndByte < upperBoundByte).
      upperBoundByte = nextLineEndByte

  # Second phase: Start at the beginning of our window, and just do a linear search through
  # each line. Since we know that the first line matching our prefix is somewhere in our window,
  # we are guaranteed to find it before we reach the end of the window. Since we know that our
  # window is relatively small (its midpoint is either on the last line or the second-to-last
  # line), we can also be reasonably confident that this linear pass will not take too long.
  #
  # TODO: Of course on some inputs this will perform very badly.
  #       In particular if your file mostly has very short lines, but has a few very, very long
  #       lines, then you might get to the second phase while still having a window with very
  #       many lines and then doing a linear search on those might take a long time.
  inputFile.setFilePos(lowerBoundByte)
  while (lowerBoundByte < upperBoundByte):
    let nextLine = inputFile.readLine()
    if nextLine.startsWith(prefix):
      break # Found the first matching line
    lowerBoundByte = inputFile.getFilePos()

  return lowerBoundByte

proc main() =
  let args = os.commandLineParams()
  if (len(args) != 2) or (args[0] == "--help"):
    stderr.write("presearch: Search a file (whose lines are sorted) for lines with a particular prefix")
    stderr.write("")
    stderr.write("Usage: presearch <input file path> <prefix>")
    return

  let inputPath = args[0]
  let prefix = args[1]

  var inputFile: File
  try:
    inputFile = open(inputPath)
  except IOError:
    stderr.write("Failed to open input file at: " & inputPath & "\n    " & getCurrentExceptionMsg())
    return

  let inputFileInfo = getFileInfo(inputFile)
  if isatty(stdout):
    echo("Searching '" & inputPath & "' for lines with prefix: '" & prefix & "'")
    echo("")

  let firstMatchOffset = offsetOfFirstMatchingLine(inputFile, input_file_info.size, prefix)

  var matchFound = false
  inputFile.setFilePos(firstMatchOffset)
  while not inputFile.endOfFile():
    let nextLine = inputFile.readLine()
    if not nextLine.startsWith(prefix):
      break

    matchFound = true
    if isatty(stdout):
      stdout.setForeGroundColor(fgCyan)
      stdout.write(prefix)
      stdout.resetAttributes()
      stdout.writeLine(nextLine[len(prefix)..^1])
    else:
      echo nextLine

  inputFile.close()

  if isatty(stdout) and not matchFound:
    echo("No matching lines found")

main()
