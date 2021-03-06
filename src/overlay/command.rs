// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// This file is part of cancer.
//
// cancer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cancer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cancer.  If not, see <http://www.gnu.org/licenses/>.

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Command {
	None,
	Exit,
	Move(Move),
	Scroll(Scroll),
	Select(Select),
	Copy(String),
	Paste(String),
	Hint(Hint),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Scroll {
	Up(u32),
	Down(u32),
	PageUp(u32),
	PageDown(u32),
	Begin,
	End,
	To(u32),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Move {
	Left(u32),
	Right(u32),
	Up(u32),
	Down(u32),
	Start,
	End,
	To(u32, u32),
	Word(Word),
	Until(Until),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Word {
	Next(u32),
	Previous(u32),
	NextEnd(u32),
	PreviousEnd(u32),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Until {
	Next(u32, String),
	Previous(u32, String),

	NextBefore(u32, String),
	PreviousBefore(u32, String),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Select {
	Normal,
	Block,
	Line,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Hint {
	Start,
	Pick(char),
	Open,
	Copy(String),
}
