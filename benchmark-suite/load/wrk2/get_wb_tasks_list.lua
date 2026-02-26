request = function()
  return wrk.format("GET", "/wb/tasks?params=%5B%22open%22%2C20%2C0%5D&row_schema=1")
end
