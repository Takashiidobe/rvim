# The Greeter class
class Greeter
  def initialize(name)
    @name = name.to_upcase
  end

  def salute
    puts "Hello #{@name}!"
  end
end

g = Greeter.new('world')
g.salute
